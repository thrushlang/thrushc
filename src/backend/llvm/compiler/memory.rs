#![allow(clippy::enum_variant_names)]

use inkwell::{
    AddressSpace,
    context::Context,
    module::Module,
    targets::TargetData,
    types::{BasicType, BasicTypeEnum},
    values::IntValue,
};

use crate::{
    backend::llvm::compiler::typegen,
    standard::logging::{self, LoggingType},
    types::frontend::lexer::types::ThrushType,
};

use inkwell::{
    builder::Builder,
    values::{BasicValue, BasicValueEnum, PointerValue},
};

use super::{context::LLVMCodeGenContext, valuegen};

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
    LowLevelInstruction {
        value: BasicValueEnum<'ctx>,
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
    LowLevelInstruction,
}

#[derive(Debug, Clone, Copy)]
pub enum LLVMAllocationSite {
    Heap,
    Stack,
    Static,
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
                        kind.is_probably_heap_allocated(llvm_context, context.get_target_data()),
                    );

                    self::store_anon(context, ptr_allocated, value);

                    value = ptr_allocated.into();
                }

                Self::Parameter { value, kind }
            }
            SymbolToAllocate::LowLevelInstruction => Self::LowLevelInstruction { value, kind },
        }
    }

    pub fn load(&self, context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

        let thrush_type: &ThrushType = self.get_type();

        if thrush_type.is_ptr_type() {
            return self.get_value();
        }

        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);
        let preferred_memory_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        match self {
            Self::Local { ptr, kind } => {
                if kind.is_probably_heap_allocated(llvm_context, target_data) {
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

                    if kind.is_probably_heap_allocated(llvm_context, target_data) {
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

            Self::LowLevelInstruction { value, .. } => *value,
        }
    }

    pub fn store(&self, context: &LLVMCodeGenContext<'_, 'ctx>, new_value: BasicValueEnum<'ctx>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

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

            Self::LowLevelInstruction { value, .. } if value.is_pointer_value() => {
                if let Ok(store) = llvm_builder.build_store(value.into_pointer_value(), new_value) {
                    let _ = store.set_alignment(preferred_memory_alignment);
                }
            }

            _ => (),
        }
    }

    pub fn raw_load(&self) -> PointerValue<'ctx> {
        match self {
            Self::Local { ptr, .. } => *ptr,
            Self::Constant { ptr, .. } => *ptr,
            Self::LowLevelInstruction { value, .. } => value.into_pointer_value(),
            Self::Parameter { value, .. } => value.into_pointer_value(),
        }
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
                    .build_in_bounds_gep(
                        typegen::generate_subtype(context, kind),
                        *ptr,
                        indexes,
                        "",
                    )
                    .unwrap()
            },
            Self::Parameter { value, kind } | Self::LowLevelInstruction { value, kind } => {
                if value.is_pointer_value() {
                    return unsafe {
                        builder
                            .build_in_bounds_gep(
                                typegen::generate_subtype(context, kind),
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
                .build_struct_gep(typegen::generate_subtype(context, kind), *ptr, index, "")
                .unwrap(),
            Self::Parameter { value, kind } | Self::LowLevelInstruction { value, kind } => {
                if value.is_pointer_value() {
                    return builder
                        .build_struct_gep(
                            typegen::generate_subtype(context, kind),
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
                        LoggingType::Panic,
                        "Built-in sizeof!(), cannot be get size of an function parameter.",
                    );

                    unreachable!()
                })
                .into(),

            Self::LowLevelInstruction { value, .. } => value
                .get_type()
                .size_of()
                .unwrap_or_else(|| {
                    logging::log(
                        LoggingType::Panic,
                        "Built-in sizeof!(), cannot be get size of an Low Level Instruction.",
                    );

                    unreachable!()
                })
                .into(),
        }
    }

    pub fn take(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } | Self::Constant { ptr, .. } => (*ptr).into(),
            Self::Parameter { value, .. } | Self::LowLevelInstruction { value, .. } => *value,
        }
    }

    pub fn get_type(&self) -> &'ctx ThrushType {
        match self {
            Self::Local { kind, .. } => kind,
            Self::Constant { kind, .. } => kind,
            Self::Parameter { kind, .. } => kind,
            Self::LowLevelInstruction { kind, .. } => kind,
        }
    }

    pub fn get_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } => (*ptr).into(),
            Self::Constant { ptr, .. } => (*ptr).into(),
            Self::Parameter { value, .. } => *value,
            Self::LowLevelInstruction { value, .. } => *value,
        }
    }
}

pub fn store_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, '_>,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
) {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let target_data: &TargetData = context.get_target_data();

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

    let preferred_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&llvm_type);

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

pub fn alloc_anon<'ctx>(
    site: LLVMAllocationSite,
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let preferred_memory_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&llvm_type);

    match site {
        LLVMAllocationSite::Stack => {
            if let Ok(ptr) = llvm_builder.build_alloca(llvm_type, "") {
                if let Some(instruction) = ptr.as_instruction() {
                    let _ = instruction.set_alignment(preferred_memory_alignment);
                }

                return ptr;
            }

            logging::log(
                LoggingType::Panic,
                &format!("Cannot assign type to stack: '{}'.", kind),
            );

            unreachable!()
        }
        LLVMAllocationSite::Heap => {
            if let Ok(ptr) = llvm_builder.build_malloc(llvm_type, "") {
                return ptr;
            }

            logging::log(
                LoggingType::Panic,
                &format!("Cannot assign type to heap: '{}'.", kind),
            );

            unreachable!()
        }
        LLVMAllocationSite::Static => llvm_module
            .add_global(llvm_type, Some(AddressSpace::default()), "")
            .as_pointer_value(),
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

    let llvm_preferred_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&llvm_type);

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
