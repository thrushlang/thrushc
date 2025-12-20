use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::traits::TypeCodeLocation;
use crate::front_end::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::FunctionType;
use inkwell::types::IntType;

use std::path::PathBuf;

#[inline]
pub fn generate_fn_type<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    parameters: &[Ast],
    is_var_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    for parameter in parameters {
        match parameter {
            Ast::FunctionParameter { kind, .. }
            | Ast::IntrinsicParameter { kind, .. }
            | Ast::AssemblerFunctionParameter { kind, .. } => {
                parameters_types.push(self::generate(context, kind).into());
            }

            _ => {}
        }
    }

    if kind.is_void_type() {
        return llvm_context
            .void_type()
            .fn_type(&parameters_types, is_var_args);
    }

    self::generate(context, kind).fn_type(&parameters_types, is_var_args)
}

#[inline]
pub fn generate_function_type_from_type<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    parameters: &[Type],
    is_var_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter_type| {
        parameters_types.push(self::generate(context, parameter_type).into());
    });

    if kind.is_void_type() {
        return llvm_context
            .void_type()
            .fn_type(&parameters_types, is_var_args);
    }

    self::generate(context, kind).fn_type(&parameters_types, is_var_args)
}

#[inline]
pub fn generate<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
) -> BasicTypeEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        t if t.is_integer_type() || t.is_char_type() || t.is_bool_type() => {
            self::generate_integer_type(context, kind).into()
        }

        t if t.is_float_type() => self::generate_float_type(context, kind).into(),

        t if t.is_ptr_like_type() => llvm_context.ptr_type(AddressSpace::default()).into(),

        Type::Const(any, ..) => self::generate(context, any),

        Type::Struct(_, fields, modificator, ..) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            let packed: bool = modificator.llvm().is_packed();

            fields.iter().for_each(|field| {
                field_types.push(self::generate(context, field));
            });

            llvm_context.struct_type(&field_types, packed).into()
        }

        Type::FixedArray(kind, size, ..) => {
            let arraytype: BasicTypeEnum = self::generate(context, kind);
            arraytype.array_type(*size).into()
        }

        any => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as a type!", any),
            any.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

#[inline]
fn generate_integer_type<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
) -> IntType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        Type::S8(..) | Type::U8(..) | Type::Char(..) => llvm_context.i8_type(),
        Type::S16(..) | Type::U16(..) => llvm_context.i16_type(),
        Type::S32(..) | Type::U32(..) => llvm_context.i32_type(),
        Type::S64(..) | Type::U64(..) => llvm_context.i64_type(),
        Type::U128(..) => llvm_context.i128_type(),
        Type::USize(..) | Type::SSize(..) => {
            llvm_context.ptr_sized_int_type(context.get_target_data(), None)
        }

        Type::Bool(..) => llvm_context.bool_type(),
        Type::Const(any, ..) => self::generate_integer_type(context, any),

        any => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as a integer type!", any),
            any.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

#[inline]
fn generate_float_type<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
) -> FloatType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        Type::F32(..) => llvm_context.f32_type(),
        Type::F64(..) => llvm_context.f64_type(),
        Type::F128(..) => llvm_context.f128_type(),
        Type::FX8680(..) => llvm_context.x86_f80_type(),
        Type::FPPC128(..) => llvm_context.ppc_f128_type(),

        Type::Const(any, ..) => self::generate_float_type(context, any),

        any => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as a float type!", any),
            any.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

#[inline]
pub fn generate_gep<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
) -> BasicTypeEnum<'ctx> {
    match kind {
        Type::Const(subtype, ..) => self::generate_gep(context, subtype),
        Type::Array(subtype, ..) => self::generate(context, subtype),
        Type::Ptr(Some(subtype), ..) => self::generate(context, subtype),

        _ => self::generate(context, kind),
    }
}

#[inline]
pub fn generate_local<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    value: Option<&Ast>,
) -> BasicTypeEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        Type::Const(subtype, ..) => self::generate_local(context, subtype, value),
        Type::Array(subtype, ..) if matches!(value, Some(Ast::Array { .. })) => {
            if let Some(Ast::Array { items, span, .. }) = value {
                self::generate_local(
                    context,
                    &Type::FixedArray(subtype.clone(), items.len() as u32, *span),
                    value,
                )
            } else {
                llvm_context.ptr_type(AddressSpace::default()).into()
            }
        }

        _ => self::generate(context, kind),
    }
}
