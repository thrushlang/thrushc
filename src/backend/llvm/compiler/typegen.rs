use crate::middle::instruction::Instruction;
use crate::middle::types::Type;

use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};

use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, FloatType, IntType},
};

#[inline]
pub fn type_int_to_llvm_int_type<'ctx>(context: &'ctx Context, kind: &Type) -> IntType<'ctx> {
    match kind {
        Type::S8 | Type::U8 | Type::Char => context.i8_type(),
        Type::S16 | Type::U16 => context.i16_type(),
        Type::S32 | Type::U32 => context.i32_type(),
        Type::S64 | Type::U64 => context.i64_type(),
        Type::Bool => context.bool_type(),
        _ => unreachable!(),
    }
}

#[inline]
pub fn type_float_to_llvm_float_type<'ctx>(context: &'ctx Context, kind: &Type) -> FloatType<'ctx> {
    match kind {
        Type::F32 => context.f32_type(),
        Type::F64 => context.f64_type(),
        _ => unreachable!(),
    }
}

pub fn function_type<'ctx>(
    context: &'ctx Context,
    kind: &Type,
    parameters: &[Instruction],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter| {
        if let Instruction::FunctionParameter { kind, .. } = parameter {
            parameters_types.push(generate_type(context, kind).into());
        }
    });

    match kind {
        Type::S8 | Type::U8 | Type::Char => {
            context.i8_type().fn_type(&parameters_types, ignore_args)
        }
        Type::S16 | Type::U16 => context.i16_type().fn_type(&parameters_types, ignore_args),
        Type::S32 | Type::U32 => context.i32_type().fn_type(&parameters_types, ignore_args),
        Type::S64 | Type::U64 => context.i64_type().fn_type(&parameters_types, ignore_args),
        Type::Str => context
            .ptr_type(AddressSpace::default())
            .fn_type(&parameters_types, ignore_args),

        Type::Address => generate_type(context, kind).fn_type(&parameters_types, ignore_args),
        Type::Ptr(_) => generate_type(context, kind).fn_type(&parameters_types, ignore_args),
        Type::Struct(_) => generate_type(context, kind).fn_type(&parameters_types, ignore_args),

        Type::Bool => context.bool_type().fn_type(&parameters_types, ignore_args),
        Type::F32 => context.f32_type().fn_type(&parameters_types, ignore_args),
        Type::F64 => context.f64_type().fn_type(&parameters_types, ignore_args),
        Type::Void => context.void_type().fn_type(&parameters_types, ignore_args),
    }
}

pub fn generate_type<'ctx>(context: &'ctx Context, kind: &Type) -> BasicTypeEnum<'ctx> {
    match kind {
        kind if kind.is_bool_type() || kind.is_integer_type() || kind.is_char_type() => {
            type_int_to_llvm_int_type(context, kind).into()
        }
        kind if kind.is_float_type() => type_float_to_llvm_float_type(context, kind).into(),
        Type::Ptr(_) | Type::Address => context.ptr_type(AddressSpace::default()).into(),
        kind if kind.is_str_type() => context
            .struct_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                ],
                false,
            )
            .into(),
        Type::Struct(fields) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            fields.iter().for_each(|field| {
                field_types.push(generate_type(context, field));
            });

            context.struct_type(&field_types, false).into()
        }

        _ => unreachable!(),
    }
}
