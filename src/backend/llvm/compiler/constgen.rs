use std::{fmt::Display, sync::Arc};

use inkwell::{context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        constants::{
            self,
            arrays::farray,
            binaryop, casts,
            generation::{floatgen, intgen},
            unaryop,
        },
        constgen,
        context::LLVMCodeGenContext,
        expressions, typegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::{
        types::ast::Ast,
        typesystem::{traits::TypeStructExtensions, types::Type},
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
        } => self::compile_int(context, kind, *value, *signed, cast),

        // Character literal compilation
        Ast::Char { byte, .. } => self::compile_char(context, *byte),

        // Floating-point constant handling
        Ast::Float {
            value,
            kind,
            signed,
            ..
        } => self::compile_float(context, kind, *value, *signed, cast),

        // Boolean true/false cases
        Ast::Boolean { value, .. } => self::compile_boolean(context, *value),

        // Fixed-size array
        Ast::FixedArray { items, .. } => farray::constant_fixed_array(context, items, cast),

        // String literal compilation
        Ast::Str { bytes, .. } => expressions::string::compile_str(context, bytes).into(),

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
                return binaryop::integer::compile(context, (left, operator, right), cast);
            }

            if binaryop_type.is_bool_type() {
                return binaryop::boolean::compile(context, (left, operator, right), cast);
            }

            if binaryop_type.is_float_type() {
                return binaryop::float::compile(context, (left, operator, right), cast);
            }

            if binaryop_type.is_ptr_type() {
                return binaryop::pointer::compile(context, (left, operator, right));
            }

            self::codegen_abort("Cannot perform constant binary expression.");
        }

        // Unary operation dispatch
        Ast::UnaryOp {
            operator,
            expression,
            kind,
            ..
        } => unaryop::compile(context, (operator, kind, expression), cast),

        // Fallback for unsupported AST nodes
        _ => {
            self::codegen_abort("Cannot perform constant expression.");
        }
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

fn compile_as<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let value_type: &Type = from.get_type_unwrapped();
    let value: BasicValueEnum = constgen::compile(context, from, value_type);

    casts::try_one(context, value, value_type, cast)
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    context.get_table().get_symbol(name).get_value()
}

fn compile_int<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    value: u64,
    signed: bool,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let int: BasicValueEnum =
        intgen::const_int(context.get_llvm_context(), kind, value, signed).into();

    let cast: BasicTypeEnum = typegen::generate_subtype_with_all(context.get_llvm_context(), cast);

    constants::casts::numeric::numeric_cast(int, cast, signed)
}

fn compile_float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    value: f64,
    signed: bool,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let float: BasicValueEnum =
        floatgen::const_float(context.get_llvm_context(), kind, value, signed).into();

    let cast: BasicTypeEnum = typegen::generate_subtype_with_all(context.get_llvm_context(), cast);

    constants::casts::numeric::numeric_cast(float, cast, signed)
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

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
