#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::generation::{floatgen, intgen, structgen};
use crate::backend::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backend::llvm::compiler::statements::lli;
use crate::backend::llvm::compiler::{
    binaryop, builtins, cast, codegen, expressions, indexes, ptrgen,
};

use crate::backend::types::traits::AssemblerFunctionExtensions;

use crate::backend::types::LLVMEitherExpression;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::traits::AstExtensions;
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::traits::LLVMTypeExtensions;
use crate::frontend::typesystem::types::Type;

use inkwell::types::{BasicTypeEnum, FunctionType, PointerType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue, StructValue,
};
use inkwell::{AddressSpace, InlineAsmDialect};
use inkwell::{builder::Builder, context::Context};

use std::fmt::Display;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        // Literal Expressions
        Ast::NullPtr { .. } => self::compile_null_ptr(context),
        Ast::Str { bytes, kind, .. } => self::compile_string(context, bytes, kind),
        Ast::Float {
            kind,
            value,
            signed,
            ..
        } => self::compile_float(context, kind, *value, *signed, cast),
        Ast::Integer {
            kind,
            value,
            signed,
            ..
        } => self::compile_integer(context, kind, *value, *signed, cast),
        Ast::Char { byte, .. } => self::compile_char(context, *byte),
        Ast::Boolean { value, .. } => self::compile_boolean(context, *value),

        // Function and Built-in Calls
        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => self::compile_function_call(context, name, args, kind, cast),

        // Compiles a sizeof operation
        Ast::SizeOf { sizeof, .. } => builtins::sizeof::compile(context, sizeof, cast),

        // Expressions
        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast),
        Ast::BinaryOp {
            left,
            operator,
            right,
            kind: binaryop_type,
            ..
        } => self::compile_binary_op(context, left, operator, right, binaryop_type, cast),
        Ast::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } => expressions::unaryop::compile(context, (operator, kind, expression), cast),

        // Symbol/Property Access
        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => self::compile_reference(context, name),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source,
            indexes,
            kind,
            ..
        } => expressions::property::compile_property_value(context, source, indexes, kind),

        // Memory Access Operations
        // Compiles an indexing operation (e.g., array access)
        Ast::Index {
            source, indexes, ..
        } => self::compile_index(context, source, indexes),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref { value, kind, .. } => self::compile_deref(context, value, kind, cast),

        // Array Operations
        // Compiles a fixed-size array
        Ast::FixedArray { items, kind, .. } => {
            expressions::farray::compile(context, items, kind, cast)
        }

        // Compiles a dynamic array
        Ast::Array { items, kind, .. } => expressions::array::compile(context, items, kind, cast),

        // Compiles a struct constructor
        Ast::Constructor { args, kind, .. } => structgen::compile(context, args, kind, cast),

        // Compiles a type cast operation
        Ast::As { from, cast, .. } => self::compile_cast(context, from, cast),

        // Low-Level Operations
        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => self::compile_inline_asm(context, assembler, constraints, args, kind, attributes),

        // Low-Level Operations
        ast if ast.is_lli() => lli::compile_advanced(context, expr, cast),

        // Fallback, Unknown expressions or statements
        what => {
            self::codegen_abort(format!(
                "Failed to compile. Unknown expression or statement '{:?}'.",
                what
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_function_call<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    args: &'ctx [Ast],
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let function: (FunctionValue, &[Type], u32) = context.get_table().get_function(name);

    let (llvm_function, function_arg_types, function_convention) =
        (function.0, function.1, function.2);

    let llvm_builder: &Builder = context.get_llvm_builder();

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .enumerate()
        .map(|(i, expr)| {
            let cast: Option<&Type> = function_arg_types.get(i);

            codegen::compile_expr(context, expr, cast).into()
        })
        .collect();

    let fn_value: BasicValueEnum = match llvm_builder.build_call(llvm_function, &compiled_args, "")
    {
        Ok(call) => {
            call.set_call_convention(function_convention);
            if !kind.is_void_type() {
                call.try_as_basic_value().left().unwrap_or_else(|| {
                    self::codegen_abort(format!("Function call '{}' returned no value.", name));
                    self::compile_null_ptr(context)
                })
            } else {
                self::compile_null_ptr(context)
            }
        }
        Err(_) => {
            self::codegen_abort(format!("Failed to generate call to function '{}'.", name));
            self::compile_null_ptr(context)
        }
    };

    cast::try_cast(context, cast, kind, fn_value).unwrap_or(fn_value)
}

fn compile_binary_op<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: &'ctx Ast,
    operator: &'ctx TokenType,
    right: &'ctx Ast,
    binaryop_type: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match binaryop_type {
        t if t.is_float_type() => binaryop::float::compile(context, (left, operator, right), cast),
        t if t.is_integer_type() => {
            binaryop::integer::compile(context, (left, operator, right), cast)
        }
        t if t.is_bool_type() => binaryop::boolean::compile(context, (left, operator, right), cast),
        t if t.is_ptr_type() => binaryop::pointer::compile(context, (left, operator, right)),

        _ => {
            self::codegen_abort(format!(
                "Invalid type '{}' for binary operation",
                binaryop_type
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let from_type: &Type = from.get_type_unwrapped();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if from_type.is_str_type() && cast.is_ptr_type() {
        let val: BasicValueEnum = ptrgen::compile(context, from, None);

        if val.is_pointer_value() {
            let raw_str_ptr: PointerValue = val.into_pointer_value();
            let str_loaded: BasicValueEnum = memory::load_anon(context, raw_str_ptr, from_type);
            let str_structure: StructValue = str_loaded.into_struct_value();

            match llvm_builder.build_extract_value(str_structure, 0, "") {
                Ok(cstr) => {
                    let to = typegen::generate_type(llvm_context, cast).into_pointer_type();
                    match llvm_builder.build_pointer_cast(cstr.into_pointer_value(), to, "") {
                        Ok(casted_ptr) => return casted_ptr.into(),
                        Err(_) => self::codegen_abort(format!(
                            "Failed to cast string pointer in '{}'.",
                            from
                        )),
                    }
                }
                Err(_) => {
                    self::codegen_abort(format!("Failed to extract string value in '{}'.", from))
                }
            }
        } else {
            let str_structure: StructValue = val.into_struct_value();

            match llvm_builder.build_extract_value(str_structure, 0, "") {
                Ok(cstr) => {
                    let to = typegen::generate_type(llvm_context, cast).into_pointer_type();
                    match llvm_builder.build_pointer_cast(cstr.into_pointer_value(), to, "") {
                        Ok(casted_ptr) => return casted_ptr.into(),
                        Err(_) => self::codegen_abort(format!(
                            "Failed to cast string pointer in '{}'.",
                            from
                        )),
                    }
                }
                Err(_) => {
                    self::codegen_abort(format!("Failed to extract string value in '{}'.", from))
                }
            }
        }
    } else if cast.llvm_is_ptr_type() {
        let val: BasicValueEnum = ptrgen::compile(context, from, None);

        if val.is_pointer_value() {
            let to: PointerType = typegen::generate_type(llvm_context, cast).into_pointer_type();
            match llvm_builder.build_pointer_cast(val.into_pointer_value(), to, "") {
                Ok(casted_ptr) => return casted_ptr.into(),
                Err(_) => self::codegen_abort(format!("Failed to cast pointer in '{}'.", from)),
            }
        }
    } else {
        let val: BasicValueEnum = self::compile(context, from, None);
        let target_type: BasicTypeEnum = typegen::generate_type(llvm_context, cast);

        if from_type.llvm_is_same_bit_size(context, cast) {
            match llvm_builder.build_bit_cast(val, target_type, "") {
                Ok(casted_value) => return casted_value,
                Err(_) => self::codegen_abort(format!(
                    "Failed bit cast from '{}' to '{}'.",
                    from_type, cast
                )),
            }
        }

        if val.is_int_value() && target_type.is_int_type() {
            match llvm_builder.build_int_cast(val.into_int_value(), target_type.into_int_type(), "")
            {
                Ok(casted_value) => return casted_value.into(),
                Err(_) => self::codegen_abort(format!(
                    "Failed integer cast from '{}' to '{}'.",
                    from_type, cast
                )),
            }
        }

        if val.is_float_value() && target_type.is_float_type() {
            match llvm_builder.build_float_cast(
                val.into_float_value(),
                target_type.into_float_type(),
                "",
            ) {
                Ok(casted_value) => return casted_value.into(),
                Err(_) => self::codegen_abort(format!(
                    "Failed float cast from '{}' to '{}'.",
                    from_type, cast
                )),
            }
        }
    }

    self::codegen_abort(format!(
        "Unsupported cast from '{}' to '{}'.",
        from_type, cast
    ));

    self::compile_null_ptr(context)
}

fn compile_deref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx Ast,
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let val: BasicValueEnum = self::compile(context, value, Some(kind));

    let deref_value: BasicValueEnum = if val.is_pointer_value() {
        memory::load_anon(context, val.into_pointer_value(), kind)
    } else {
        val
    };

    cast::try_cast(context, cast, kind, deref_value).unwrap_or(deref_value)
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    context.get_table().get_symbol(name).load(context)
}

fn compile_inline_asm<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    assembler: &str,
    constraints: &str,
    args: &'ctx [Ast],
    kind: &Type,
    attributes: &ThrushAttributes,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_type: FunctionType = typegen::function_type(context, kind, args, false);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .map(|arg| self::compile(context, arg, None).into())
        .collect();

    let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();

    for attr in attributes {
        if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attr {
            syntax = str::to_inline_assembler_dialect(new_syntax);
        }
    }

    let fn_inline_assembler: PointerValue = llvm_context.create_inline_asm(
        asm_function_type,
        assembler.to_string(),
        constraints.to_string(),
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    match llvm_builder.build_indirect_call(
        asm_function_type,
        fn_inline_assembler,
        &compiled_args,
        "",
    ) {
        Ok(call) if !kind.is_void_type() => call.try_as_basic_value().left().unwrap_or_else(|| {
            self::codegen_abort("Inline assembler returned no value.");

            self::compile_null_ptr(context)
        }),
        Ok(_) => self::compile_null_ptr(context),
        Err(_) => {
            self::codegen_abort("Failed to build inline assembler.");

            self::compile_null_ptr(context)
        }
    }
}

fn compile_index<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx LLVMEitherExpression<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);
            let symbol_type: &Type = symbol.get_type();

            let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, symbol_type);

            symbol
                .gep(llvm_context, llvm_builder, &ordered_indexes)
                .into()
        }
        (_, Some(expr)) => {
            let expr_ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();
            let expr_type: &Type = expr.get_type_unwrapped();

            let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, expr_type);

            memory::gep_anon(context, expr_ptr, expr_type, &ordered_indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid index target in expression.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_string<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    bytes: &'ctx [u8],
    kind: &Type,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = expressions::string::compile_str_constant(context, bytes);
    memory::load_anon(context, ptr, kind)
}

fn compile_float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx Type,
    value: f64,
    signed: bool,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let float: BasicValueEnum =
        floatgen::float(context.get_llvm_context(), kind, value, signed).into();

    cast::try_cast(context, cast, kind, float).unwrap_or(float)
}

fn compile_integer<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx Type,
    value: u64,
    signed: bool,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let int: BasicValueEnum = intgen::int(context.get_llvm_context(), kind, value, signed).into();

    cast::try_cast(context, cast, kind, int).unwrap_or(int)
}

fn compile_char<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>, byte: u64) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .i8_type()
        .const_int(byte, false)
        .into()
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

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
