#![allow(clippy::enum_variant_names)]

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    context::Context,
    module::{Linkage, Module},
    targets::TargetData,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::{FunctionValue, IntValue},
};

use crate::{
    backend::llvm::{self, compiler::typegen},
    middle::types::frontend::lexer::types::ThrushType,
    standard::logging,
};

use inkwell::{
    builder::Builder,
    values::{BasicValue, BasicValueEnum, PointerValue},
};

use super::{context::LLVMCodeGenContext, utils, valuegen};

#[derive(Debug, Clone)]
pub enum SymbolAllocated<'ctx> {
    Local {
        ptr: PointerValue<'ctx>,
        kind: &'ctx ThrushType,
    },
    Constant {
        ptr: PointerValue<'ctx>,
        kind: &'ctx ThrushType,
    },
    Parameter {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx ThrushType,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolToAllocate {
    Local,
    Constant,
    Parameter,
}

#[derive(Debug, Clone, Copy)]
pub enum AllocSite {
    Heap,
    Stack,
}

impl<'ctx> SymbolAllocated<'ctx> {
    pub fn new(
        context: &LLVMCodeGenContext<'_, 'ctx>,
        allocate: SymbolToAllocate,
        value: BasicValueEnum<'ctx>,
        kind: &'ctx ThrushType,
    ) -> Self {
        match allocate {
            SymbolToAllocate::Local => Self::Local {
                ptr: value.into_pointer_value(),
                kind,
            },
            SymbolToAllocate::Constant => Self::Constant {
                ptr: value.into_pointer_value(),
                kind,
            },
            SymbolToAllocate::Parameter => {
                let llvm_builder: &Builder = context.get_llvm_builder();
                let llvm_context: &Context = context.get_llvm_context();

                let mut value: BasicValueEnum<'ctx> = value;

                if !kind.is_mut_type() {
                    let ptr_allocated: PointerValue = valuegen::alloc(
                        llvm_context,
                        llvm_builder,
                        kind,
                        kind.is_heap_allocated(llvm_context, &context.target_data),
                    );

                    self::store_anon(context, ptr_allocated, value);

                    value = ptr_allocated.into();
                }

                Self::Parameter { value, kind }
            }
        }
    }

    pub fn load(&self, context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = &context.target_data;

        let thrush_type: &ThrushType = self.get_type();

        if thrush_type.is_ptr_type() {
            return self.get_value();
        }

        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);
        let preferred_memory_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        match self {
            Self::Local { ptr, kind } => {
                if kind.is_heap_allocated(llvm_context, target_data) {
                    if context.get_position().in_call() {
                        let loaded_value: BasicValueEnum =
                            llvm_builder.build_load(llvm_type, *ptr, "").unwrap();

                        if let Some(load_instruction) = loaded_value.as_instruction_value() {
                            let _ = load_instruction.set_alignment(preferred_memory_alignment);
                        }

                        return loaded_value;
                    }

                    return (*ptr).into();
                }

                let loaded_value: BasicValueEnum =
                    llvm_builder.build_load(llvm_type, *ptr, "").unwrap();

                if let Some(load_instruction) = loaded_value.as_instruction_value() {
                    let _ = load_instruction.set_alignment(preferred_memory_alignment);
                }

                loaded_value
            }
            Self::Parameter { value, kind } => {
                if value.is_pointer_value() {
                    let ptr: PointerValue = value.into_pointer_value();

                    if kind.is_heap_allocated(llvm_context, target_data) {
                        if context.get_position().in_call() {
                            let loaded_value: BasicValueEnum =
                                llvm_builder.build_load(llvm_type, ptr, "").unwrap();

                            if let Some(load_instruction) = value.as_instruction_value() {
                                let _ = load_instruction.set_alignment(preferred_memory_alignment);
                            }

                            return loaded_value;
                        }

                        return *value;
                    }

                    let loaded_value: BasicValueEnum =
                        llvm_builder.build_load(llvm_type, ptr, "").unwrap();

                    if let Some(load_instruction) = loaded_value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(preferred_memory_alignment);
                    }

                    return loaded_value;
                }

                *value
            }

            Self::Constant { ptr, .. } => {
                let loaded_value: BasicValueEnum =
                    llvm_builder.build_load(llvm_type, *ptr, "").unwrap();

                if let Some(load_instruction) = loaded_value.as_instruction_value() {
                    let _ = load_instruction.set_alignment(preferred_memory_alignment);
                }

                loaded_value
            }
        }
    }

    pub fn store(&self, context: &LLVMCodeGenContext<'_, 'ctx>, new_value: BasicValueEnum<'ctx>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = &context.target_data;

        let thrush_type: &ThrushType = self.get_type();
        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);

        let preferred_memory_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        match self {
            Self::Local { ptr, .. } => {
                if let Ok(store) = llvm_builder.build_store(*ptr, new_value) {
                    let _ = store.set_alignment(preferred_memory_alignment);
                }
            }

            Self::Parameter { value, .. } if value.is_pointer_value() => {
                if let Ok(store) = llvm_builder.build_store(value.into_pointer_value(), new_value) {
                    let _ = store.set_alignment(preferred_memory_alignment);
                }
            }

            _ => (),
        }
    }

    pub fn dealloc(&self, context: &LLVMCodeGenContext<'_, '_>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = &context.target_data;

        match self {
            Self::Local { ptr, kind, .. } if kind.is_heap_allocated(llvm_context, target_data) => {
                if kind.has_any_recursive_type() {
                    if let Some(last_block) = llvm_builder.get_insert_block() {
                        let recursive_paths: Vec<Vec<u32>> = kind.get_recursive_type_paths();

                        let deallocator: FunctionValue =
                            self::create_deallocator(context, kind, &recursive_paths);

                        llvm_builder.position_at_end(last_block);

                        let _ = llvm_builder.build_call(deallocator, &[(*ptr).into()], "");

                        llvm_builder.position_at_end(last_block);

                        return;
                    }
                }

                let _ = llvm_builder.build_free(*ptr);
            }

            Self::Parameter { value, kind, .. }
                if kind.is_heap_allocated(llvm_context, target_data)
                    && value.is_pointer_value() =>
            {
                let ptr: PointerValue = value.into_pointer_value();

                if kind.has_any_recursive_type() {
                    if let Some(last_block) = llvm_builder.get_insert_block() {
                        let recursive_paths: Vec<Vec<u32>> = kind.get_recursive_type_paths();

                        let deallocator: FunctionValue =
                            self.create_deallocator(context, kind, &recursive_paths);

                        llvm_builder.position_at_end(last_block);

                        let _ = llvm_builder.build_call(deallocator, &[ptr.into()], "");

                        llvm_builder.position_at_end(last_block);

                        return;
                    }
                }

                let _ = llvm_builder.build_free(ptr);
            }

            _ => (),
        }
    }

    fn create_deallocator(
        &self,
        context: &'ctx LLVMCodeGenContext<'_, '_>,
        kind: &'ctx ThrushType,
        paths: &[Vec<u32>],
    ) -> FunctionValue<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_module: &Module = context.get_llvm_module();

        let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

        let deallocator_type: FunctionType = llvm_context.void_type().fn_type(
            &[llvm_context.ptr_type(AddressSpace::default()).into()],
            false,
        );

        let randomness_length: usize = utils::generate_random_range(40);

        let deallocator_name: String =
            utils::generate_random_function_name("thrush_deallocator", randomness_length);

        let deallocator: FunctionValue = llvm_module.add_function(
            &deallocator_name,
            deallocator_type,
            Some(Linkage::LinkerPrivate),
        );

        let deallocator_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

        let recursive_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");
        let out_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

        llvm_builder.position_at_end(deallocator_block);

        if let Some(param) = deallocator.get_first_param() {
            let ptr: PointerValue = param.into_pointer_value();

            let is_null_ptr: IntValue = llvm_builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    ptr,
                    llvm_context.ptr_type(AddressSpace::default()).const_null(),
                    "",
                )
                .unwrap();

            let _ = llvm_builder.build_conditional_branch(is_null_ptr, recursive_block, out_block);

            llvm_builder.position_at_end(recursive_block);

            let memory_calculations: Vec<PointerValue> = paths
                .iter()
                .map(|path| {
                    let mut path_transformed: Vec<IntValue> = path
                        .iter()
                        .map(|path| llvm_context.i32_type().const_int(*path as u64, false))
                        .collect();

                    path_transformed.insert(0, llvm_context.i32_type().const_int(0, false));

                    let memory_calculated: PointerValue = unsafe {
                        llvm_builder
                            .build_in_bounds_gep(llvm_type, ptr, &path_transformed, "")
                            .unwrap()
                    };

                    self::load_anon(context, &ThrushType::Ptr(None), memory_calculated)
                        .into_pointer_value()
                })
                .collect();

            memory_calculations.iter().for_each(|ptr| {
                let _ = llvm_builder.build_call(deallocator, &[(*ptr).into()], "");
            });

            let _ = llvm_builder.build_free(ptr);

            let _ = llvm_builder.build_unconditional_branch(out_block);

            llvm_builder.position_at_end(out_block);

            let _ = llvm_builder.build_return(None);
        }

        deallocator
    }

    pub fn gep(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        indexes: &[IntValue<'ctx>],
    ) -> PointerValue<'ctx> {
        match self {
            Self::Local { ptr, kind } | Self::Constant { ptr, kind } => unsafe {
                builder
                    .build_in_bounds_gep(typegen::generate_type(context, kind), *ptr, indexes, "")
                    .unwrap()
            },
            Self::Parameter { value, kind } => {
                if value.is_pointer_value() {
                    return unsafe {
                        builder
                            .build_in_bounds_gep(
                                typegen::generate_type(context, kind),
                                (*value).into_pointer_value(),
                                indexes,
                                "",
                            )
                            .unwrap()
                    };
                }

                unreachable!()
            }
        }
    }

    pub fn gep_struct(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        index: u32,
    ) -> PointerValue<'ctx> {
        match self {
            Self::Local { ptr, kind } | Self::Constant { ptr, kind } => builder
                .build_struct_gep(typegen::generate_type(context, kind), *ptr, index, "")
                .unwrap(),
            Self::Parameter { value, kind } => {
                if value.is_pointer_value() {
                    return builder
                        .build_struct_gep(
                            typegen::generate_type(context, kind),
                            (*value).into_pointer_value(),
                            index,
                            "",
                        )
                        .unwrap();
                }

                unreachable!()
            }
        }
    }

    pub fn get_size(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } => ptr.get_type().size_of().into(),
            Self::Constant { ptr, .. } => ptr.get_type().size_of().into(),
            Self::Parameter { value, .. } => value
                .get_type()
                .size_of()
                .unwrap_or_else(|| {
                    logging::log(
                        logging::LoggingType::Panic,
                        "built-in sizeof!(), cannot be get size of an function parameter.",
                    );

                    unreachable!()
                })
                .into(),
        }
    }

    pub fn take(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } | Self::Constant { ptr, .. } => (*ptr).into(),
            Self::Parameter { value, .. } => *value,
        }
    }

    pub fn get_type(&self) -> &'ctx ThrushType {
        match self {
            Self::Local { kind, .. } => kind,
            Self::Constant { kind, .. } => kind,
            Self::Parameter { kind, .. } => kind,
        }
    }

    pub fn get_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } => (*ptr).into(),
            Self::Constant { ptr, .. } => (*ptr).into(),
            Self::Parameter { value, .. } => *value,
        }
    }
}

pub fn create_deallocator<'ctx>(
    context: &'ctx LLVMCodeGenContext<'_, '_>,
    kind: &'ctx ThrushType,
    paths: &[Vec<u32>],
) -> FunctionValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_module: &Module = context.get_llvm_module();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

    let deallocator_type: FunctionType = llvm_context.void_type().fn_type(
        &[llvm_context.ptr_type(AddressSpace::default()).into()],
        false,
    );

    let randomness_length: usize = utils::generate_random_range(40);
    let deallocator_name: String =
        utils::generate_random_function_name("thrush_deallocator", randomness_length);

    let deallocator: FunctionValue = llvm_module.add_function(
        &deallocator_name,
        deallocator_type,
        Some(Linkage::LinkerPrivate),
    );

    let entry_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");
    let recursive_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");
    let free_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");
    let out_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

    llvm_builder.position_at_end(entry_block);

    if let Some(param) = deallocator.get_first_param() {
        let ptr: PointerValue = param.into_pointer_value();

        let is_null_ptr: IntValue = llvm_builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                ptr,
                llvm_context.ptr_type(AddressSpace::default()).const_null(),
                "",
            )
            .unwrap();

        let _ = llvm_builder.build_conditional_branch(is_null_ptr, recursive_block, out_block);

        llvm_builder.position_at_end(recursive_block);

        for path in paths.iter() {
            let check_null_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

            let call_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

            let continue_block: BasicBlock = llvm_context.append_basic_block(deallocator, "");

            let _ = llvm_builder.build_unconditional_branch(check_null_block);

            llvm_builder.position_at_end(check_null_block);

            let mut path_transformed: Vec<IntValue> = path
                .iter()
                .map(|path| llvm_context.i32_type().const_int(*path as u64, false))
                .collect();

            path_transformed.insert(0, llvm_context.i32_type().const_int(0, false));

            let memory_calculated: PointerValue = unsafe {
                llvm_builder
                    .build_in_bounds_gep(llvm_type, ptr, &path_transformed, "")
                    .unwrap()
            };

            let loaded_ptr: PointerValue =
                self::load_anon(context, &ThrushType::Ptr(None), memory_calculated)
                    .into_pointer_value();

            let is_null_loaded: IntValue = llvm_builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    loaded_ptr,
                    llvm_context.ptr_type(AddressSpace::default()).const_null(),
                    "",
                )
                .unwrap();

            let _ =
                llvm_builder.build_conditional_branch(is_null_loaded, call_block, continue_block);

            llvm_builder.position_at_end(call_block);

            let _ = llvm_builder.build_call(deallocator, &[loaded_ptr.into()], "");
            let _ = llvm_builder.build_unconditional_branch(continue_block);

            llvm_builder.position_at_end(continue_block);
        }

        let _ = llvm_builder.build_unconditional_branch(free_block);
        llvm_builder.position_at_end(free_block);

        let _ = llvm_builder.build_free(ptr);

        let _ = llvm_builder.build_unconditional_branch(out_block);
        llvm_builder.position_at_end(out_block);

        let _ = llvm_builder.build_return(None);
    }

    deallocator
}

pub fn store_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, '_>,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
) {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let target_data: &TargetData = &context.target_data;

    let preferred_memory_alignment: u32 = target_data.get_preferred_alignment(&ptr.get_type());

    if let Ok(store) = llvm_builder.build_store(ptr, value) {
        let _ = store.set_alignment(preferred_memory_alignment);
    }
}

pub fn load_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    ptr: PointerValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let preferred_alignment: u32 = context.target_data.get_preferred_alignment(&llvm_type);

    let loaded_value: BasicValueEnum = llvm_builder.build_load(llvm_type, ptr, "").unwrap();

    if let Some(load_instruction) = loaded_value.as_instruction_value() {
        let _ = load_instruction.set_alignment(preferred_alignment);
    }

    loaded_value
}

pub fn load_maybe<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    value: BasicValueEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    if value.is_pointer_value() {
        let new_value: BasicValueEnum = self::load_anon(context, kind, value.into_pointer_value());
        return new_value;
    }

    value
}

pub fn alloc<'ctx>(
    site: AllocSite,
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

    match site {
        AllocSite::Stack => llvm_builder.build_alloca(llvm_type, "").unwrap(),
        AllocSite::Heap => llvm_builder.build_malloc(llvm_type, "").unwrap(),
    }
}

pub fn memcpy<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    dst: PointerValue<'ctx>,
    src: PointerValue<'ctx>,
    kind: &ThrushType,
) {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

    let llvm_preferred_alignment: u32 = context.target_data.get_preferred_alignment(&llvm_type);

    if let Some(size) = llvm_type.size_of() {
        let _ = llvm_builder.build_memcpy(
            dst,
            llvm_preferred_alignment * 2,
            src,
            llvm_preferred_alignment,
            size,
        );
    }
}
