use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};

use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, FloatType, IntType},
};

use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::parser::stmts::stmt::ThrushStatement;

use super::context::LLVMCodeGenContext;

#[inline]
pub fn thrush_integer_to_llvm_type<'ctx>(
    llvm_context: &'ctx Context,
    kind: &ThrushType,
) -> IntType<'ctx> {
    match kind {
        ThrushType::S8 | ThrushType::U8 | ThrushType::Char => llvm_context.i8_type(),
        ThrushType::S16 | ThrushType::U16 => llvm_context.i16_type(),
        ThrushType::S32 | ThrushType::U32 => llvm_context.i32_type(),
        ThrushType::S64 | ThrushType::U64 => llvm_context.i64_type(),
        ThrushType::Bool => llvm_context.bool_type(),

        ThrushType::Mut(any) => thrush_integer_to_llvm_type(llvm_context, any),

        _ => unreachable!(),
    }
}

#[inline]
pub fn type_float_to_llvm_float_type<'ctx>(
    llvm_context: &'ctx Context,
    kind: &ThrushType,
) -> FloatType<'ctx> {
    match kind {
        ThrushType::F32 => llvm_context.f32_type(),
        ThrushType::F64 => llvm_context.f64_type(),

        ThrushType::Mut(any) => type_float_to_llvm_float_type(llvm_context, any),

        _ => unreachable!(),
    }
}

pub fn function_type<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    parameters: &[ThrushStatement],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter| {
        if let ThrushStatement::FunctionParameter { kind, .. } = parameter {
            let llvm_type: BasicMetadataTypeEnum = self::generate_type(llvm_context, kind).into();
            parameters_types.push(llvm_type);
        }
    });

    if kind.is_void_type() {
        return llvm_context
            .void_type()
            .fn_type(&parameters_types, ignore_args);
    }

    self::generate_type(llvm_context, kind).fn_type(&parameters_types, ignore_args)
}

pub fn generate_type<'ctx>(llvm_context: &'ctx Context, kind: &ThrushType) -> BasicTypeEnum<'ctx> {
    match kind {
        kind if kind.is_bool_type() || kind.is_integer_type() || kind.is_char_type() => {
            thrush_integer_to_llvm_type(llvm_context, kind).into()
        }

        kind if kind.is_float_type() => type_float_to_llvm_float_type(llvm_context, kind).into(),

        ThrushType::Str => llvm_context
            .struct_type(
                &[
                    llvm_context.ptr_type(AddressSpace::default()).into(),
                    llvm_context.i64_type().into(),
                ],
                false,
            )
            .into(),

        ThrushType::Ptr(_) | ThrushType::Addr | ThrushType::Mut(..) => {
            llvm_context.ptr_type(AddressSpace::default()).into()
        }

        ThrushType::Struct(_, fields) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            fields.iter().for_each(|field| {
                field_types.push(self::generate_type(llvm_context, field));
            });

            llvm_context.struct_type(&field_types, false).into()
        }

        ThrushType::FixedArray(kind, size) => {
            let arraytype: BasicTypeEnum = self::generate_type(llvm_context, kind);
            arraytype.array_type(*size).into()
        }

        any => {
            logging::log(
                LoggingType::Bug,
                &format!("Unable to create a LLVM Type from '{}' type.", any),
            );

            unreachable!()
        }
    }
}

pub fn generate_subtype<'ctx>(
    llvm_context: &'ctx Context,
    kind: &ThrushType,
) -> BasicTypeEnum<'ctx> {
    match kind {
        ThrushType::Ptr(Some(subtype)) => generate_subtype(llvm_context, subtype),
        ThrushType::Mut(subtype) => generate_subtype(llvm_context, subtype),

        _ => generate_type(llvm_context, kind),
    }
}
