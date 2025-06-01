use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};

use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, FloatType, IntType},
};

use crate::types::frontend::lexer::types::ThrushType;
use crate::types::frontend::parser::stmts::stmt::ThrushStatement;

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
        ThrushType::Mut(subtype) => thrush_integer_to_llvm_type(llvm_context, subtype),

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
        ThrushType::Mut(subtype) => type_float_to_llvm_float_type(llvm_context, subtype),
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
            let llvm_type: BasicMetadataTypeEnum = generate_type(llvm_context, kind).into();
            parameters_types.push(llvm_type);
        }
    });

    match kind {
        ThrushType::S8 | ThrushType::U8 | ThrushType::Char => llvm_context
            .i8_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::S16 | ThrushType::U16 => llvm_context
            .i16_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::S32 | ThrushType::U32 => llvm_context
            .i32_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::S64 | ThrushType::U64 => llvm_context
            .i64_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::Str => llvm_context
            .ptr_type(AddressSpace::default())
            .fn_type(&parameters_types, ignore_args),

        ThrushType::Address => {
            generate_type(llvm_context, kind).fn_type(&parameters_types, ignore_args)
        }
        ThrushType::Ptr(_) => {
            generate_type(llvm_context, kind).fn_type(&parameters_types, ignore_args)
        }
        ThrushType::Struct(..) => {
            let is_heap_allocated_type: bool =
                kind.is_probably_heap_allocated(llvm_context, context.get_target_data());

            if is_heap_allocated_type {
                return llvm_context
                    .ptr_type(AddressSpace::default())
                    .fn_type(&parameters_types, ignore_args);
            }

            generate_type(llvm_context, kind).fn_type(&parameters_types, ignore_args)
        }
        ThrushType::Mut(..) => {
            generate_type(llvm_context, kind).fn_type(&parameters_types, ignore_args)
        }

        ThrushType::Bool => llvm_context
            .bool_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::F32 => llvm_context
            .f32_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::F64 => llvm_context
            .f64_type()
            .fn_type(&parameters_types, ignore_args),
        ThrushType::Void => llvm_context
            .void_type()
            .fn_type(&parameters_types, ignore_args),
    }
}

pub fn generate_type<'ctx>(llvm_context: &'ctx Context, kind: &ThrushType) -> BasicTypeEnum<'ctx> {
    match kind {
        kind if kind.is_bool_type() || kind.is_integer_type() || kind.is_char_type() => {
            thrush_integer_to_llvm_type(llvm_context, kind).into()
        }
        kind if kind.is_float_type() => type_float_to_llvm_float_type(llvm_context, kind).into(),
        ThrushType::Ptr(_) | ThrushType::Address | ThrushType::Mut(..) => {
            llvm_context.ptr_type(AddressSpace::default()).into()
        }
        kind if kind.is_str_type() => llvm_context
            .struct_type(
                &[
                    llvm_context.ptr_type(AddressSpace::default()).into(),
                    llvm_context.i64_type().into(),
                ],
                false,
            )
            .into(),
        ThrushType::Struct(_, fields) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            fields.iter().for_each(|field| {
                field_types.push(generate_subtype(llvm_context, field));
            });

            llvm_context.struct_type(&field_types, false).into()
        }

        _ => unreachable!(),
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
