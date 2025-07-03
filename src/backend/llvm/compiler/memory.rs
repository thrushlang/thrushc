#![allow(clippy::enum_variant_names)]

use std::fmt::Display;

use inkwell::{
    AddressSpace,
    context::Context,
    module::Module,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{IntValue, StructValue},
};

use crate::{
    backend::llvm::compiler::typegen,
    core::console::logging::{self, LoggingType},
    frontend::types::lexer::Type,
};

use inkwell::{
    builder::Builder,
    values::{BasicValue, BasicValueEnum, PointerValue},
};

use super::context::LLVMCodeGenContext;

#[derive(Debug, Clone, Copy)]
pub enum SymbolAllocated<'ctx> {
    Local {
        ptr: PointerValue<'ctx>,
        kind: &'ctx Type,
    },
    Constant {
        ptr: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
    },
    LowLevelInstruction {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
    },
    Parameter {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolToAllocate {
    Local,
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
    pub fn new(allocate: SymbolToAllocate, kind: &'ctx Type, value: BasicValueEnum<'ctx>) -> Self {
        match allocate {
            SymbolToAllocate::Local => Self::Local {
                ptr: value.into_pointer_value(),
                kind,
            },
            SymbolToAllocate::Parameter => Self::Parameter { value, kind },
            SymbolToAllocate::LowLevelInstruction => Self::LowLevelInstruction { value, kind },
        }
    }

    pub fn new_constant(
        ptr: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
    ) -> Self {
        Self::Constant {
            ptr: ptr.into_pointer_value(),
            value,
            kind,
        }
    }

    pub fn load(&self, context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

        let thrush_type: &Type = self.get_type();

        if thrush_type.is_ptr_type() {
            return self.get_ptr().into();
        }

        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);
        let mem_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        match self {
            Self::Local { ptr, .. } => {
                if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, *ptr, "") {
                    if let Some(load_instruction) = loaded_value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(mem_alignment);
                    }

                    return loaded_value;
                }

                self::codegen_abort("Unable to load value at memory manipulation.");
                unreachable!()
            }
            Self::Parameter { value, .. } => {
                if value.is_pointer_value() {
                    let ptr: PointerValue = value.into_pointer_value();

                    if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, ptr, "") {
                        if let Some(load_instruction) = loaded_value.as_instruction_value() {
                            let _ = load_instruction.set_alignment(mem_alignment);
                        }

                        return loaded_value;
                    }

                    self::codegen_abort("Unable to load value at memory manipulation.");
                    unreachable!()
                }

                *value
            }

            Self::Constant { ptr, .. } => {
                if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, *ptr, "") {
                    if let Some(load_instruction) = loaded_value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(mem_alignment);
                    }

                    return loaded_value;
                }

                self::codegen_abort("Unable to load value at memory manipulation.");
                unreachable!()
            }

            Self::LowLevelInstruction { value, .. } => *value,
        }
    }

    pub fn store(&self, context: &LLVMCodeGenContext<'_, 'ctx>, new_value: BasicValueEnum<'ctx>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

        let thrush_type: &Type = self.get_type();
        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);

        let mem_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        match self {
            Self::Local { ptr, .. } => {
                if let Ok(store) = llvm_builder.build_store(*ptr, new_value) {
                    let _ = store.set_alignment(mem_alignment);
                }
            }

            Self::Parameter { value, .. } if value.is_pointer_value() => {
                if let Ok(store) = llvm_builder.build_store(value.into_pointer_value(), new_value) {
                    let _ = store.set_alignment(mem_alignment);
                }
            }

            Self::LowLevelInstruction { value, .. } if value.is_pointer_value() => {
                if let Ok(store) = llvm_builder.build_store(value.into_pointer_value(), new_value) {
                    let _ = store.set_alignment(mem_alignment);
                }
            }

            _ => (),
        }
    }

    pub fn gep(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        indexes: &[IntValue<'ctx>],
    ) -> PointerValue<'ctx> {
        match self {
            Self::Local { ptr, kind } | Self::Constant { ptr, kind, .. } => unsafe {
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

                self::codegen_abort("Unable to calculate pointer position at memory manipulation.");

                unreachable!()
            }
        }
    }

    pub fn extract_value(&self, builder: &Builder<'ctx>, index: u32) -> BasicValueEnum<'ctx> {
        match self {
            Self::Parameter { value, .. } | Self::LowLevelInstruction { value, .. } => {
                if value.is_struct_value() {
                    let struct_value: StructValue = value.into_struct_value();
                    if let Ok(extracted_value) =
                        builder.build_extract_value(struct_value, index, "")
                    {
                        return extracted_value;
                    }
                }

                self::codegen_abort(
                    "Unable to get a value of an structure at memory manipulation.",
                );

                unreachable!()
            }

            _ => {
                self::codegen_abort(
                    "Unable to get a value of an structure at memory manipulation.",
                );

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
            Self::Local { ptr, kind } | Self::Constant { ptr, kind, .. } => builder
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

                self::codegen_abort(
                    "Unable to get struct element pointer position at memory manipulation.",
                );

                unreachable!()
            }
        }
    }

    pub fn get_type(&self) -> &'ctx Type {
        match self {
            Self::Local { kind, .. } => kind,
            Self::Constant { kind, .. } => kind,
            Self::Parameter { kind, .. } => kind,
            Self::LowLevelInstruction { kind, .. } => kind,
        }
    }

    pub fn get_ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Local { ptr, .. } => *ptr,
            Self::Constant { ptr, .. } => *ptr,
            Self::Parameter { value, .. } => value.into_pointer_value(),
            Self::LowLevelInstruction { value, .. } => value.into_pointer_value(),
        }
    }

    pub fn get_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } => (*ptr).into(),
            Self::Constant { value, .. } => *value,
            Self::Parameter { value, .. } => *value,
            Self::LowLevelInstruction { value, .. } => *value,
        }
    }

    pub fn is_pointer(&self) -> bool {
        match self {
            Self::Local { .. } => true,
            Self::Constant { .. } => true,
            Self::Parameter { value, .. } => value.is_pointer_value(),
            Self::LowLevelInstruction { value, .. } => value.is_pointer_value(),
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

    let mem_alignment: u32 = target_data.get_preferred_alignment(&value.get_type());

    if let Ok(store) = llvm_builder.build_store(ptr, value) {
        let _ = store.set_alignment(mem_alignment);
    }
}

pub fn load_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    kind: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

    let preferred_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&ptr.get_type());

    let loaded_value: BasicValueEnum = llvm_builder.build_load(llvm_type, ptr, "").unwrap();

    if let Some(load_instruction) = loaded_value.as_instruction_value() {
        let _ = load_instruction.set_alignment(preferred_alignment);
    }

    loaded_value
}

pub fn alloc_anon<'ctx>(
    site: LLVMAllocationSite,
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    without_subtyped: bool,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = if without_subtyped {
        typegen::generate_type(llvm_context, kind)
    } else {
        typegen::generate_subtype(llvm_context, kind)
    };

    let mem_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&llvm_type);

    match site {
        LLVMAllocationSite::Stack => {
            if let Ok(ptr) = llvm_builder.build_alloca(llvm_type, "") {
                if let Some(instruction) = ptr.as_instruction() {
                    let _ = instruction.set_alignment(mem_alignment);
                }

                return ptr;
            }

            self::codegen_abort(format!("Cannot assign type to stack: '{}'.", kind));
            unreachable!()
        }
        LLVMAllocationSite::Heap => {
            if let Ok(ptr) = llvm_builder.build_malloc(llvm_type, "") {
                return ptr;
            }

            self::codegen_abort(format!("Cannot assign type to heap: '{}'.", kind));
            unreachable!()
        }
        LLVMAllocationSite::Static => llvm_module
            .add_global(llvm_type, Some(AddressSpace::default()), "")
            .as_pointer_value(),
    }
}

pub fn gep_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    kind: &Type,
    indexes: &[IntValue<'ctx>],
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let Ok(ptr) = unsafe {
        llvm_builder.build_gep(
            typegen::generate_subtype(llvm_context, kind),
            ptr,
            indexes,
            "",
        )
    } {
        return ptr;
    }

    self::codegen_abort("Unable to get pointer element at memory manipulation.");
    unreachable!()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
