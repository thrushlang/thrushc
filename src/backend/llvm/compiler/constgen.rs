use std::{fmt::Display, sync::Arc};

use inkwell::{AddressSpace, context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        binaryop, constants, constgen, context::LLVMCodeGenContext, floatgen, intgen, string,
        typegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::{
        types::ast::Ast,
        typesystem::{
            traits::{LLVMTypeExtensions, TypeStructExtensions},
            types::Type,
        },
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ast: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    match ast {
        // Handle integer literals
        Ast::Integer {
            value,
            kind,
            signed,
            ..
        } => self::compile_integer(context, kind, *value, *signed, cast),

        // Character literal compilation
        Ast::Char { byte, .. } => self::compile_char(context, *byte),

        // Floating-point constant handling
        Ast::Float {
            value,
            kind,
            signed,
            ..
        } => self::compile_float(context, kind, *value, *signed),

        // Boolean true/false cases
        Ast::Boolean { value, .. } => self::compile_boolean(context, *value),

        // Fixed-size array initialization
        Ast::FixedArray { items, .. } => self::constant_fixed_array(context, cast, items),

        // String literal compilation
        Ast::Str { bytes, .. } => string::compile_str(context, bytes).into(),

        // Struct constructor handling
        Ast::Constructor { args, kind, .. } => {
            let fields: Vec<&Ast> = args.iter().map(|raw_arg| &raw_arg.1).collect();
            self::constant_struct(context, kind, fields)
        }

        // Type casting operations
        Ast::As { from, cast, .. } => self::compile_as(context, from, cast),

        // Variable reference resolution
        Ast::Reference { name, .. } => self::compile_reference(context, name),

        // Grouped expression compilation
        Ast::Group { expression, .. } => self::compile(context, expression, cast),

        // Binary operation dispatch
        Ast::BinaryOp {
            left,
            operator,
            right,
            kind: binaryop_type,
            ..
        } => {
            if binaryop_type.is_integer_type() {
                return binaryop::constants::integer::const_integer_binaryop(
                    context,
                    (left, operator, right),
                    cast,
                );
            }

            if binaryop_type.is_bool_type() {
                return binaryop::constants::boolean::const_bool_binaryop(
                    context,
                    (left, operator, right),
                    cast,
                );
            }

            if binaryop_type.is_float_type() {
                return binaryop::constants::float::const_float_binaryop(
                    context,
                    (left, operator, right),
                    cast,
                );
            }

            if binaryop_type.is_ptr_type() {
                return binaryop::constants::pointer::const_ptr_binaryop(
                    context,
                    (left, operator, right),
                );
            }

            self::codegen_abort("Cannot perform constant binary expression.");
            self::compile_null_ptr(context)
        }

        // Fallback for unsupported AST nodes
        _ => {
            self::codegen_abort("Cannot perform constant expression.");
            self::compile_null_ptr(context)
        }
    }
}
pub fn cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    value_type: &Type,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match (value_type, cast) {
        (from_ty, cast_ty) if from_ty.is_str_type() && cast_ty.is_ptr_type() => {
            let cast: BasicTypeEnum = typegen::generate_type(llvm_context, cast_ty);

            constants::cast::ptr_cast(context, value, cast)
        }

        (_, cast_ty) if cast_ty.is_ptr_type() || cast_ty.is_mut_type() => {
            let cast: BasicTypeEnum = typegen::generate_type(llvm_context, cast_ty);

            constants::cast::ptr_cast(context, value, cast)
        }

        (_, cast_ty) if cast_ty.is_numeric() => {
            if value_type.llvm_is_same_bit_size(context, cast_ty) {
                constants::bitcast::const_numeric_bitcast_cast(context, value, cast)
            } else {
                constants::cast::numeric_cast(
                    context,
                    value,
                    cast,
                    value_type.is_signed_integer_type(),
                )
            }
        }

        _ => value,
    }
}

pub fn constant_struct<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    fields: Vec<&'ctx Ast>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let struct_fields_types: &[Arc<Type>] = kind.get_struct_fields();

    let fields: Vec<BasicValueEnum> = fields
        .iter()
        .zip(struct_fields_types)
        .map(|(field, kind)| constgen::compile(context, field, kind))
        .collect();

    llvm_context.const_struct(&fields, false).into()
}

pub fn constant_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_item_type: &Type = kind.get_fixed_array_base_type();

    let array_type: BasicTypeEnum = typegen::generate_type(llvm_context, array_item_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| {
            let value_type: &Type = item.get_type_unwrapped();
            let value: BasicValueEnum = constgen::compile(context, item, array_item_type);

            self::cast(context, value, value_type, array_item_type)
        })
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

fn compile_as<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let value_type: &Type = from.get_type_unwrapped();
    let value: BasicValueEnum = constgen::compile(context, from, value_type);

    self::cast(context, value, value_type, cast)
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    context.get_symbol(name).get_value()
}

fn compile_integer<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    value: u64,
    signed: bool,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let int: BasicValueEnum =
        intgen::integer(context.get_llvm_context(), kind, value, signed).into();

    constants::cast::numeric_cast(context, int, cast, signed)
}

fn compile_float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
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

    constants::cast::numeric_cast(context, float, kind, signed)
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
