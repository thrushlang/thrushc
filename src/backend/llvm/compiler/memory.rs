#![allow(clippy::enum_variant_names)]

use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum},
    values::IntValue,
};

use crate::{backend::llvm::compiler::typegen, common::logging, middle::types::Type};

use inkwell::{
    builder::Builder,
    values::{BasicValue, BasicValueEnum, InstructionValue, PointerValue},
};

#[derive(Debug, Clone)]
pub enum SymbolAllocated<'ctx> {
    Local {
        ptr: PointerValue<'ctx>,
        kind: &'ctx Type,
    },
    Constant {
        ptr: PointerValue<'ctx>,
        kind: &'ctx Type,
    },
    Parameter {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
    },
}

impl<'ctx> SymbolAllocated<'ctx> {
    pub fn new_constant(ptr: PointerValue<'ctx>, kind: &'ctx Type) -> Self {
        Self::Constant { ptr, kind }
    }

    pub fn new_local(ptr: PointerValue<'ctx>, kind: &'ctx Type) -> Self {
        Self::Local { ptr, kind }
    }

    pub fn new_parameter(value: BasicValueEnum<'ctx>, kind: &'ctx Type) -> Self {
        Self::Parameter { value, kind }
    }

    pub fn load(&self, context: &'ctx Context, builder: &Builder<'ctx>) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, kind } => {
                if kind.is_stack_allocated() {
                    let value: BasicValueEnum = builder
                        .build_load(typegen::generate_typed_pointer(context, kind), *ptr, "")
                        .unwrap();

                    if let Some(load_instruction) = value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(8);
                    }

                    return value;
                }

                (*ptr).into()
            }
            Self::Parameter { value, kind } => {
                if value.is_pointer_value() {
                    let ptr: PointerValue = value.into_pointer_value();

                    if kind.is_stack_allocated() {
                        let value: BasicValueEnum = builder
                            .build_load(typegen::generate_typed_pointer(context, kind), ptr, "")
                            .unwrap();

                        if let Some(load_instruction) = value.as_instruction_value() {
                            let _ = load_instruction.set_alignment(8);
                        }

                        return value;
                    }

                    return ptr.into();
                }

                *value
            }

            Self::Constant { ptr, kind } => {
                if kind.is_stack_allocated() {
                    let value: BasicValueEnum = builder
                        .build_load(typegen::generate_typed_pointer(context, kind), *ptr, "")
                        .unwrap();

                    if let Some(load_instruction) = value.as_instruction_value() {
                        let _ = load_instruction.set_alignment(8);
                    }

                    return value;
                }

                (*ptr).into()
            }
        }
    }

    pub fn dealloc(&self, builder: &Builder<'ctx>) {
        match self {
            Self::Local { ptr, kind, .. } if kind.is_recursive_type() => {
                let _ = builder.build_free(*ptr);
            }

            Self::Parameter { value, kind, .. }
                if kind.is_recursive_type() && value.is_pointer_value() =>
            {
                let _ = builder.build_free(value.into_pointer_value());
            }

            _ => (),
        }
    }

    pub fn store(&self, builder: &Builder<'ctx>, value: BasicValueEnum<'ctx>) {
        match self {
            Self::Local { ptr, .. } => {
                let store: InstructionValue = builder.build_store(*ptr, value).unwrap();
                let _ = store.set_alignment(8);
            }

            Self::Parameter { value: ptr, .. } if ptr.is_pointer_value() => {
                let store: InstructionValue = builder
                    .build_store(ptr.into_pointer_value(), value)
                    .unwrap();
                let _ = store.set_alignment(8);
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

    pub fn get_type(&self) -> &'ctx Type {
        match self {
            Self::Local { kind, .. }
            | Self::Parameter { kind, .. }
            | Self::Constant { kind, .. } => kind,
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
                        "built-in sizeof!(), cannot be get size of an function parameter. ",
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

pub fn load_anon<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &Type,
    ptr: PointerValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    if kind.is_stack_allocated() {
        let value: BasicValueEnum = builder
            .build_load(typegen::generate_typed_pointer(context, kind), ptr, "")
            .unwrap();

        if let Some(load_instruction) = value.as_instruction_value() {
            let _ = load_instruction.set_alignment(8);
        }

        return value;
    }

    ptr.into()
}

pub fn store_anon<'ctx>(
    builder: &Builder<'ctx>,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
) {
    let store: InstructionValue = builder.build_store(ptr, value).unwrap();
    let _ = store.set_alignment(8);
}
