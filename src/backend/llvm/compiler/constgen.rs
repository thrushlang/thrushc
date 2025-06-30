use std::{fmt::Display, sync::Arc};

use inkwell::{AddressSpace, context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        constgen, context::LLVMCodeGenContext, floatgen, intgen, string, typegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::{
        ast::Ast,
        lexer::{ThrushType, traits::ThrushTypeStructExtensions},
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    ast: &Ast,
) -> BasicValueEnum<'ctx> {
    match ast {
        Ast::Integer { value, signed, .. } => self::compile_integer(context, kind, *value, *signed),

        Ast::Char { byte, .. } => self::compile_char(context, *byte),

        Ast::Float {
            kind,
            value,
            signed,
            ..
        } => self::compile_float(context, kind, *value, *signed),

        Ast::Boolean { value, .. } => self::compile_boolean(context, *value),

        Ast::FixedArray { items, .. } => self::constant_fixed_array(context, kind, items),

        Ast::Str { bytes, .. } => string::compile_str(context, bytes).into(),

        Ast::Constructor { args, kind, .. } => {
            let fields: Vec<&Ast> = args.iter().map(|raw_arg| &raw_arg.1).collect();
            self::constant_struct(context, kind, fields)
        }

        _ => {
            self::codegen_abort("Cannot perform constant expression.");
            self::compile_null_ptr(context)
        }
    }
}

pub fn constant_struct<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    fields: Vec<&Ast>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let struct_fields_types: &[Arc<ThrushType>] = kind.get_struct_fields();

    let fields: Vec<BasicValueEnum> = fields
        .iter()
        .zip(struct_fields_types)
        .map(|(field, kind)| constgen::compile(context, kind, field))
        .collect();

    llvm_context.const_struct(&fields, false).into()
}

pub fn constant_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &[Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_item_type: &ThrushType = kind.get_fixed_array_base_type();

    let array_type: BasicTypeEnum = typegen::generate_type(llvm_context, array_item_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| constgen::compile(context, array_item_type, item))
        .collect();

    match array_type {
        t if t.is_int_type() => t
            .into_int_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_int_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_float_type() => t
            .into_float_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_float_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_array_type() => t
            .into_array_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_array_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_struct_type() => t
            .into_struct_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_struct_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_pointer_type() => t
            .into_pointer_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_pointer_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        _ => {
            self::codegen_abort(format!(
                "Incompatible type '{}' for constant array",
                array_item_type
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_integer<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    value: u64,
    signed: bool,
) -> BasicValueEnum<'ctx> {
    let int: BasicValueEnum =
        intgen::integer(context.get_llvm_context(), kind, value, signed).into();

    int
}

fn compile_float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    value: f64,
    signed: bool,
) -> BasicValueEnum<'ctx> {
    let float: BasicValueEnum = floatgen::float(
        context.get_llvm_builder(),
        context.get_llvm_context(),
        kind,
        value,
        signed,
    )
    .into();

    float
}

fn compile_boolean<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    value: u64,
) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .bool_type()
        .const_int(value, false)
        .into()
}

fn compile_char<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>, byte: u64) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .i8_type()
        .const_int(byte, false)
        .into()
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
