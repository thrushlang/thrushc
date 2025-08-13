use std::fmt::Display;

use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};

use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, FloatType, IntType},
};

use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::traits::LLVMTypeExtensions;
use crate::frontend::typesystem::types::Type;

use super::context::LLVMCodeGenContext;

pub fn function_type<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    parameters: &[Ast],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter| {
        if let Ast::FunctionParameter { kind, .. } = parameter {
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

#[inline]
pub fn generate_type<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> BasicTypeEnum<'ctx> {
    match kind {
        t if t.llvm_is_int_type() => self::integer_to_llvm_type(llvm_context, kind).into(),
        t if t.llvm_is_float_type() => self::float_to_llvm_type(llvm_context, kind).into(),
        t if t.llvm_is_ptr_type() => llvm_context.ptr_type(AddressSpace::default()).into(),

        Type::Const(any) => self::generate_type(llvm_context, any),

        Type::Str => llvm_context
            .struct_type(
                &[
                    llvm_context.ptr_type(AddressSpace::default()).into(),
                    llvm_context.i32_type().into(),
                ],
                false,
            )
            .into(),

        Type::Struct(_, fields) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            fields.iter().for_each(|field| {
                field_types.push(self::generate_type(llvm_context, field));
            });

            llvm_context.struct_type(&field_types, false).into()
        }

        Type::FixedArray(kind, size) => {
            let arraytype: BasicTypeEnum = self::generate_type(llvm_context, kind);
            arraytype.array_type(*size).into()
        }

        any => {
            logging::log(
                LoggingType::BackendBug,
                &format!("Unable to create a LLVM Type from '{}' type.", any),
            );

            unreachable!()
        }
    }
}

#[inline]
pub fn integer_to_llvm_type<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> IntType<'ctx> {
    match kind {
        Type::S8 | Type::U8 | Type::Char => llvm_context.i8_type(),
        Type::S16 | Type::U16 => llvm_context.i16_type(),
        Type::S32 | Type::U32 => llvm_context.i32_type(),
        Type::S64 | Type::U64 => llvm_context.i64_type(),
        Type::Bool => llvm_context.bool_type(),

        Type::Mut(any) => self::integer_to_llvm_type(llvm_context, any),
        Type::Const(any) => self::integer_to_llvm_type(llvm_context, any),

        any => {
            self::codegen_abort(format!(
                "Unable to generate LLVM float type with type '{}'.",
                any,
            ));
        }
    }
}

#[inline]
pub fn float_to_llvm_type<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> FloatType<'ctx> {
    match kind {
        Type::F32 => llvm_context.f32_type(),
        Type::F64 => llvm_context.f64_type(),

        Type::Mut(any) => self::float_to_llvm_type(llvm_context, any),
        Type::Const(any) => self::float_to_llvm_type(llvm_context, any),

        any => {
            self::codegen_abort(format!(
                "Unable to generate LLVM float type with type '{}'.",
                any,
            ));
        }
    }
}

#[inline]
pub fn generate_subtype<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> BasicTypeEnum<'ctx> {
    match kind {
        Type::Mut(subtype) => self::generate_subtype(llvm_context, subtype),
        Type::Const(subtype) => self::generate_subtype(llvm_context, subtype),
        Type::Array(subtype, ..) => self::generate_subtype(llvm_context, subtype),

        _ => self::generate_type(llvm_context, kind),
    }
}

#[inline]
pub fn generate_subtype_with_all<'ctx>(
    llvm_context: &'ctx Context,
    kind: &Type,
) -> BasicTypeEnum<'ctx> {
    match kind {
        Type::Ptr(Some(subtype)) => self::generate_subtype_with_all(llvm_context, subtype),
        Type::Mut(subtype) => self::generate_subtype_with_all(llvm_context, subtype),
        Type::Const(subtype) => self::generate_subtype_with_all(llvm_context, subtype),
        Type::Array(subtype, ..) => self::generate_subtype_with_all(llvm_context, subtype),

        _ => self::generate_type(llvm_context, kind),
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
