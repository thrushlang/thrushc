#![allow(clippy::enum_variant_names)]

use inkwell::{
    context::Context,
    targets::TargetData,
    types::{BasicType, BasicTypeEnum},
    values::IntValue,
};

use crate::{
    backend::llvm::compiler::typegen, middle::types::frontend::lexer::types::ThrushType,
    standard::logging,
};

use inkwell::{
    builder::Builder,
    values::{BasicValue, BasicValueEnum, InstructionValue, PointerValue},
};

use super::context::LLVMCodeGenContext;

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

impl<'ctx> SymbolAllocated<'ctx> {
    pub fn new_constant(ptr: PointerValue<'ctx>, kind: &'ctx ThrushType) -> Self {
        Self::Constant { ptr, kind }
    }

    pub fn new_local(ptr: PointerValue<'ctx>, kind: &'ctx ThrushType) -> Self {
        Self::Local { ptr, kind }
    }

    pub fn new_parameter(value: BasicValueEnum<'ctx>, kind: &'ctx ThrushType) -> Self {
        Self::Parameter { value, kind }
    }

    pub fn load(&self, context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();
        let target_data: &TargetData = &context.target_data;

        match self {
            Self::Local { ptr, kind } => {
                if kind.is_ptr_type() {
                    return (*ptr).into();
                }

                let ptr_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);
                let preffered_alignment: u32 = target_data.get_preferred_alignment(&ptr_type);

                if kind.is_heap_allocated(llvm_context, target_data) {
                    if context.get_position().in_call() {
                        let value: BasicValueEnum =
                            llvm_builder.build_load(ptr_type, *ptr, "").unwrap();

                        if let Some(load_instruction) = value.as_instruction_value() {
                            let _ = load_instruction.set_alignment(preffered_alignment);
                        }

                        return value;
                    }

                    return (*ptr).into();
                }

                let value: BasicValueEnum = llvm_builder.build_load(ptr_type, *ptr, "").unwrap();

                if let Some(load_instruction) = value.as_instruction_value() {
                    let _ = load_instruction.set_alignment(preffered_alignment);
                }

                value
            }
            Self::Parameter { value, kind } => {
                if kind.is_ptr_type() {
                    return *value;
                }

                if value.is_pointer_value() {
                    let ptr: PointerValue = value.into_pointer_value();
                    let ptr_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

                    let preffered_alignment: u32 = target_data.get_preferred_alignment(&ptr_type);

                    if kind.is_heap_allocated(llvm_context, target_data) {
                        if context.get_position().in_call() {
                            let value: BasicValueEnum =
                                llvm_builder.build_load(ptr_type, ptr, "").unwrap();

                            if let Some(load_instruction) = value.as_instruction_value() {
                                let _ = load_instruction.set_alignment(preffered_alignment);
                            }

                            return value;
                        }

                        return *value;
                    }

                    let value: BasicValueEnum = llvm_builder.build_load(ptr_type, ptr, "").unwrap();

                    if let Some(load_instruction) = value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(preffered_alignment);
                    }

                    return value;
                }

                *value
            }

            Self::Constant { ptr, kind } => {
                let ptr_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);
                let preffered_alignment: u32 = target_data.get_preferred_alignment(&ptr_type);

                let value: BasicValueEnum = llvm_builder.build_load(ptr_type, *ptr, "").unwrap();

                if let Some(load_instruction) = value.as_instruction_value() {
                    let _ = load_instruction.set_alignment(preffered_alignment);
                }

                value
            }
        }
    }

    pub fn dealloc(&self, context: &LLVMCodeGenContext<'_, '_>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();
        let target_data: &TargetData = &context.target_data;

        match self {
            Self::Local { ptr, kind, .. } if kind.is_heap_allocated(llvm_context, target_data) => {
                let _ = llvm_builder.build_free(*ptr);
            }

            Self::Parameter { value, kind, .. }
                if kind.is_heap_allocated(llvm_context, target_data)
                    && value.is_pointer_value() =>
            {
                let ptr: PointerValue = value.into_pointer_value();
                let _ = llvm_builder.build_free(ptr);
            }

            _ => (),
        }
    }

    pub fn store(&self, context: &LLVMCodeGenContext<'_, 'ctx>, value: BasicValueEnum<'ctx>) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();
        let target_data: &TargetData = &context.target_data;

        match self {
            Self::Local { ptr, kind, .. } => {
                let kind: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);
                let alignment: u32 = target_data.get_preferred_alignment(&kind);

                let store: InstructionValue = llvm_builder.build_store(*ptr, value).unwrap();
                let _ = store.set_alignment(alignment);
            }

            Self::Parameter {
                value: ptr, kind, ..
            } if ptr.is_pointer_value() => {
                let kind: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);
                let alignment: u32 = target_data.get_preferred_alignment(&kind);

                let store: InstructionValue = llvm_builder
                    .build_store(ptr.into_pointer_value(), value)
                    .unwrap();

                let _ = store.set_alignment(alignment);
            }
            _ => (),
        }
    }

    pub fn take(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } | Self::Constant { ptr, .. } => (*ptr).into(),
            Self::Parameter { value, .. } => *value,
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

    pub fn get_size_of(&self) -> BasicValueEnum<'ctx> {
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
}

pub fn gep_struct_from_ptr<'ctx>(
    builder: &Builder<'ctx>,
    kind: BasicTypeEnum<'ctx>,
    ptr: PointerValue<'ctx>,
    index: u32,
) -> PointerValue<'ctx> {
    builder.build_struct_gep(kind, ptr, index, "").unwrap()
}

pub fn store_anon<'ctx>(
    builder: &Builder<'ctx>,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
) {
    let store: InstructionValue = builder.build_store(ptr, value).unwrap();
    let _ = store.set_alignment(8);
}

pub trait MemoryManagement<'ctx> {
    fn load_maybe(
        &self,
        kind: &ThrushType,
        context: &LLVMCodeGenContext<'_, 'ctx>,
    ) -> BasicValueEnum<'ctx>;
}

impl<'ctx> MemoryManagement<'ctx> for BasicValueEnum<'ctx> {
    fn load_maybe(
        &self,
        kind: &ThrushType,
        context: &LLVMCodeGenContext<'_, 'ctx>,
    ) -> BasicValueEnum<'ctx> {
        if self.is_pointer_value() {
            let new_value: BasicValueEnum = load_anon(context, kind, self.into_pointer_value());
            return new_value;
        }

        *self
    }
}

pub fn load_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    ptr: PointerValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let target_data: &TargetData = &context.target_data;

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let preffered_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

    let value: BasicValueEnum = llvm_builder.build_load(llvm_type, ptr, "").unwrap();

    if let Some(load_instruction) = value.as_instruction_value() {
        let _ = load_instruction.set_alignment(preffered_alignment);
    }

    value
}
