use crate::backend::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::traits::LLVMTypeExtensions;
use crate::frontend::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use std::fmt::Display;

use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType},
};

#[inline]
pub fn generate_fn_type<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    parameters: &[Ast],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter| {
        if let Ast::FunctionParameter { kind, .. } = parameter {
            let llvm_type: BasicMetadataTypeEnum = self::generate(llvm_context, kind).into();
            parameters_types.push(llvm_type);
        }
    });

    if kind.is_void_type() {
        return llvm_context
            .void_type()
            .fn_type(&parameters_types, ignore_args);
    }

    self::generate(llvm_context, kind).fn_type(&parameters_types, ignore_args)
}

#[inline]
pub fn generate_fn_type_from_type<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    parameters: &[Type],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    parameters.iter().for_each(|parameter_type| {
        parameters_types.push(self::generate(llvm_context, parameter_type).into());
    });

    if kind.is_void_type() {
        return llvm_context
            .void_type()
            .fn_type(&parameters_types, ignore_args);
    }

    self::generate(llvm_context, kind).fn_type(&parameters_types, ignore_args)
}

#[inline]
pub fn generate<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> BasicTypeEnum<'ctx> {
    match kind {
        t if t.llvm_is_int_type() => self::integer(llvm_context, kind).into(),
        t if t.llvm_is_float_type() => self::float(llvm_context, kind).into(),
        t if t.llvm_is_ptr_type() => llvm_context.ptr_type(AddressSpace::default()).into(),

        Type::Const(any) => self::generate(llvm_context, any),

        Type::Struct(_, fields, modificator) => {
            let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

            let packed: bool = modificator.llvm().is_packed();

            fields.iter().for_each(|field| {
                field_types.push(self::generate(llvm_context, field));
            });

            llvm_context.struct_type(&field_types, packed).into()
        }

        Type::FixedArray(kind, size) => {
            let arraytype: BasicTypeEnum = self::generate(llvm_context, kind);
            arraytype.array_type(*size).into()
        }

        any => {
            self::codegen_abort(format!("Unable to create a LLVM Type from '{}' type.", any));
        }
    }
}

#[inline]
pub fn generate_gep<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> BasicTypeEnum<'ctx> {
    match kind {
        Type::Const(subtype) => self::generate_gep(llvm_context, subtype),
        Type::Ptr(Some(subtype)) => self::generate(llvm_context, subtype),

        _ => self::generate(llvm_context, kind),
    }
}

#[inline]
pub fn generate_for_local_variable<'ctx>(
    llvm_context: &'ctx Context,
    kind: &Type,
    value: Option<&Ast>,
) -> BasicTypeEnum<'ctx> {
    match kind {
        Type::Const(subtype) => self::generate_for_local_variable(llvm_context, subtype, value),
        Type::Array(subtype) if matches!(value, Some(Ast::Array { .. })) => {
            if let Some(Ast::Array { items, .. }) = value {
                self::generate_for_local_variable(
                    llvm_context,
                    &Type::FixedArray(subtype.clone(), items.len() as u32),
                    value,
                )
            } else {
                llvm_context.ptr_type(AddressSpace::default()).into()
            }
        }

        _ => self::generate(llvm_context, kind),
    }
}

#[inline]
pub fn integer<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> IntType<'ctx> {
    match kind {
        Type::S8 | Type::U8 | Type::Char => llvm_context.i8_type(),
        Type::S16 | Type::U16 => llvm_context.i16_type(),
        Type::S32 | Type::U32 => llvm_context.i32_type(),
        Type::S64 | Type::U64 => llvm_context.i64_type(),
        Type::U128 => llvm_context.i128_type(),

        Type::Bool => llvm_context.bool_type(),

        Type::Const(any) => self::integer(llvm_context, any),

        any => {
            self::codegen_abort(format!(
                "Unable to generate LLVM float type with type '{}'.",
                any,
            ));
        }
    }
}

#[inline]
pub fn float<'ctx>(llvm_context: &'ctx Context, kind: &Type) -> FloatType<'ctx> {
    match kind {
        Type::F32 => llvm_context.f32_type(),
        Type::F64 => llvm_context.f64_type(),
        Type::F128 => llvm_context.f128_type(),
        Type::FX8680 => llvm_context.x86_f80_type(),
        Type::FPPC128 => llvm_context.ppc_f128_type(),

        Type::Const(any) => self::float(llvm_context, any),

        any => {
            self::codegen_abort(format!(
                "Unable to generate LLVM float type with type '{}'.",
                any,
            ));
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
