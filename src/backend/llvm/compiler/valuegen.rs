#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backend::llvm::compiler::{
    array, binaryop, builtins, cast, farray, floatgen, intgen, lli, ptrgen, string, unaryop,
    valuegen,
};

use crate::backend::types::LLVMEitherExpression;
use crate::backend::types::traits::AssemblerFunctionExtensions;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::lexer::traits::{
    LLVMTypeExtensions, ThrushTypeMutableExtensions, ThrushTypePointerExtensions,
};
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;

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
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match expr {
        // Literal Expressions
        // Compiles a null pointer literal
        Ast::NullPtr { .. } => self::compile_null_ptr(context),

        // Compiles a string literal
        Ast::Str { bytes, kind, .. } => self::compile_string(context, bytes, kind),

        // Compiles a floating-point literal
        Ast::Float {
            kind,
            value,
            signed,
            ..
        } => self::compile_float(context, kind, *value, *signed, cast_type),

        // Compiles an integer literal
        Ast::Integer {
            kind,
            value,
            signed,
            ..
        } => self::compile_integer(context, kind, *value, *signed, cast_type),

        // Compiles a character literal
        Ast::Char { byte, .. } => self::compile_char(context, *byte),

        // Compiles a boolean literal
        Ast::Boolean { value, .. } => self::compile_boolean(context, *value),

        // Function and Built-in Calls
        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => self::compile_function_call(context, name, args, kind, cast_type),

        // Compiles a sizeof operation
        Ast::SizeOf { sizeof, .. } => builtins::sizeof::compile(context, sizeof, cast_type),

        // Operations
        // Compiles a binary operation (e.g., a + b)
        Ast::BinaryOp {
            left,
            operator,
            right,
            kind: binaryop_type,
            ..
        } => self::compile_binary_op(context, left, operator, right, binaryop_type, cast_type),

        // Compiles a unary operation (e.g., -x)
        Ast::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } => self::compile_unary_op(context, operator, kind, expression, cast_type),

        // Symbol/Property Access
        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => self::compile_reference(context, name),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            name,
            indexes,
            kind,
            ..
        } => self::compile_property(context, name, indexes, kind),

        // Memory Access Operations
        // Compiles an indexing operation (e.g., array access)
        Ast::Index {
            index_to, indexes, ..
        } => self::compile_index(context, index_to, indexes),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref { value, kind, .. } => self::compile_deref(context, value, kind, cast_type),

        // Array Operations
        // Compiles a fixed-size array
        Ast::FixedArray { items, kind, .. } => {
            farray::compile_fixed_array(context, kind, items, cast_type)
        }

        // Compiles a dynamic array
        Ast::Array { items, kind, .. } => array::compile_array(context, kind, items, cast_type),

        // Type/Structural Operations
        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast_type),

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

        // Fallback
        // Fallback for unhandled AST variants
        _ => lli::compile(context, expr, cast_type),
    }
}

fn compile_float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx ThrushType,
    value: f64,
    signed: bool,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let mut float: BasicValueEnum = floatgen::float(
        context.get_llvm_builder(),
        context.get_llvm_context(),
        kind,
        value,
        signed,
    )
    .into();

    if let Some(cast_type) = cast_type {
        if let Some(casted_float) = cast::float(context, cast_type, kind, float) {
            float = casted_float;
        }
    }

    float
}

fn compile_integer<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx ThrushType,
    value: u64,
    signed: bool,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let mut int: BasicValueEnum =
        intgen::integer(context.get_llvm_context(), kind, value, signed).into();

    if let Some(cast_type) = cast_type {
        if let Some(casted_int) = cast::integer(context, cast_type, kind, int) {
            int = casted_int;
        }
    }

    int
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

fn compile_function_call<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    args: &'ctx [Ast],
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let function: (FunctionValue<'_>, &[ThrushType], u32) = context.get_function(name);

    let (llvm_function, function_arg_types, function_convention) =
        (function.0, function.1, function.2);

    let llvm_builder: &Builder = context.get_llvm_builder();

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let arg_cast_type: Option<&ThrushType> = function_arg_types.get(i);

            let arg: BasicValueEnum =
                if arg_cast_type.is_some_and(|t| t.is_ptr_type() || t.is_mut_type()) {
                    ptrgen::compile(context, arg, arg_cast_type)
                } else {
                    self::compile(context, arg, arg_cast_type)
                };

            arg.into()
        })
        .collect();

    let fn_value = match llvm_builder.build_call(llvm_function, &compiled_args, "") {
        Ok(call) => {
            call.set_call_convention(function_convention);
            if !kind.is_void_type() {
                call.try_as_basic_value().left().unwrap_or_else(|| {
                    self::codegen_abort(format!("Function call '{}' returned no value", name));
                    self::compile_null_ptr(context)
                })
            } else {
                self::compile_null_ptr(context)
            }
        }
        Err(_) => {
            self::codegen_abort(format!("Failed to generate call to function '{}'", name));
            self::compile_null_ptr(context)
        }
    };

    if let Some(cast) = cast_type {
        cast::try_cast(context, cast, kind, fn_value).unwrap_or(fn_value)
    } else {
        fn_value
    }
}

fn compile_binary_op<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: &'ctx Ast,
    operator: &'ctx TokenType,
    right: &'ctx Ast,
    binaryop_type: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match binaryop_type {
        t if t.is_float_type() => {
            binaryop::float::float_binaryop(context, (left, operator, right), cast_type)
        }
        t if t.is_integer_type() => {
            binaryop::integer::integer_binaryop(context, (left, operator, right), cast_type)
        }
        t if t.is_bool_type() => {
            binaryop::boolean::bool_binaryop(context, (left, operator, right), cast_type)
        }
        t if t.is_ptr_type() => binaryop::ptr::ptr_binaryop(context, (left, operator, right)),

        _ => {
            self::codegen_abort(format!(
                "Invalid type '{}' for binary operation",
                binaryop_type
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_unary_op<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    operator: &'ctx TokenType,
    kind: &'ctx ThrushType,
    expression: &'ctx Ast,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    unaryop::unary_op(context, (operator, kind, expression), cast_type)
}

fn compile_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx Ast,
    cast: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let from_type: &ThrushType = from.get_type_unwrapped();
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
                            "Failed to cast string pointer in '{}'",
                            from
                        )),
                    }
                }
                Err(_) => {
                    self::codegen_abort(format!("Failed to extract string value in '{}'", from))
                }
            }
        }
    } else if cast.is_ptr_type() || cast.is_mut_type() {
        let val: BasicValueEnum = ptrgen::compile(context, from, None);

        if val.is_pointer_value() {
            let to: PointerType = typegen::generate_type(llvm_context, cast).into_pointer_type();
            match llvm_builder.build_pointer_cast(val.into_pointer_value(), to, "") {
                Ok(casted_ptr) => return casted_ptr.into(),
                Err(_) => self::codegen_abort(format!("Failed to cast pointer in '{}'", from)),
            }
        }
    } else {
        let val: BasicValueEnum = self::compile(context, from, None);
        let target_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, cast);

        if from_type.is_same_size(context, cast) {
            match llvm_builder.build_bit_cast(val, target_type, "") {
                Ok(casted_value) => return casted_value,
                Err(_) => self::codegen_abort(format!(
                    "Failed bit cast from '{}' to '{}'",
                    from_type, cast
                )),
            }
        }
        if val.is_int_value() && target_type.is_int_type() {
            match llvm_builder.build_int_cast(val.into_int_value(), target_type.into_int_type(), "")
            {
                Ok(casted_value) => return casted_value.into(),
                Err(_) => self::codegen_abort(format!(
                    "Failed integer cast from '{}' to '{}'",
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
                    "Failed float cast from '{}' to '{}'",
                    from_type, cast
                )),
            }
        }
    }

    self::codegen_abort(format!(
        "Unsupported cast from '{}' to '{}'",
        from_type, cast
    ));

    self::compile_null_ptr(context)
}

fn compile_deref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx Ast,
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let val: BasicValueEnum = self::compile(context, value, Some(kind));

    let deref_value: BasicValueEnum = if val.is_pointer_value() {
        memory::load_anon(context, val.into_pointer_value(), kind)
    } else {
        self::codegen_abort(format!(
            "Cannot dereference non-pointer value in '{}'",
            value
        ));

        val
    };

    if let Some(cast) = cast_type {
        cast::try_cast(context, cast, kind, deref_value).unwrap_or(deref_value)
    } else {
        deref_value
    }
}

fn compile_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    indexes: &[(ThrushType, u32)],
    kind: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let symbol: SymbolAllocated = context.get_allocated_symbol(name);

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if symbol.is_pointer() {
        let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);

        for index in indexes.iter().skip(1) {
            let index_type: BasicTypeEnum = typegen::generate_type(llvm_context, &index.0);

            match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
                Ok(new_ptr) => ptr = new_ptr,
                Err(_) => {
                    self::codegen_abort(format!(
                        "Failed to access property at index {} for '{}'",
                        index.1, name
                    ));

                    return self::compile_null_ptr(context);
                }
            }
        }
        memory::load_anon(context, ptr, kind)
    } else {
        let mut value = symbol.extract_value(llvm_builder, indexes[0].1);
        for index in indexes.iter().skip(1) {
            if value.is_struct_value() {
                match llvm_builder.build_extract_value(value.into_struct_value(), index.1, "") {
                    Ok(new_value) => value = new_value,
                    Err(_) => {
                        self::codegen_abort(format!(
                            "Failed to extract value at index {} for '{}'",
                            index.1, name
                        ));

                        return self::compile_null_ptr(context);
                    }
                }
            }
        }
        value
    }
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    context.get_allocated_symbol(name).load(context)
}

fn compile_inline_asm<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    assembler: &str,
    constraints: &str,
    args: &'ctx [Ast],
    kind: &ThrushType,
    attributes: &ThrushAttributes,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_type: FunctionType = typegen::function_type(context, kind, args, false);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .map(|arg| self::compile(context, arg, None).into()) // Recursive
        .collect();

    let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();

    for attr in attributes {
        if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attr {
            syntax = str::assembler_syntax_attr_to_inline_assembler_dialect(new_syntax);
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
            self::codegen_abort("Inline assembler returned no value");

            self::compile_null_ptr(context)
        }),
        Ok(_) => self::compile_null_ptr(context),
        Err(_) => {
            self::codegen_abort("Failed to build inline assembler");

            self::compile_null_ptr(context)
        }
    }
}

fn compile_index<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    index_to: &'ctx LLVMEitherExpression<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match index_to {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_allocated_symbol(name);
            let symbol_type: &ThrushType = symbol.get_type();

            let ordered_indexes: Vec<IntValue> =
                self::compute_indexes(context, indexes, symbol_type);

            symbol
                .gep(llvm_context, llvm_builder, &ordered_indexes)
                .into()
        }
        (_, Some(expr)) => {
            let expr_ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();
            let expr_type: &ThrushType = expr.get_type_unwrapped();

            let ordered_indexes: Vec<IntValue> = self::compute_indexes(context, indexes, expr_type);

            memory::gep_anon(context, expr_ptr, expr_type, &ordered_indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid index target in expression");
            self::compile_null_ptr(context)
        }
    }
}

fn compute_indexes<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [Ast],
    kind: &'ctx ThrushType,
) -> Vec<IntValue<'ctx>> {
    let llvm_context = context.get_llvm_context();
    indexes
        .iter()
        .flat_map(|index| {
            if kind.is_fixed_array_type()
                || kind.is_mut_fixed_array_type()
                || kind.is_ptr_fixed_array_type()
            {
                let base: IntValue = intgen::integer(llvm_context, &ThrushType::U32, 0, false);
                let depth: IntValue =
                    valuegen::compile(context, index, Some(&ThrushType::U32)).into_int_value();
                vec![base, depth]
            } else {
                let depth =
                    valuegen::compile(context, index, Some(&ThrushType::U64)).into_int_value();
                vec![depth]
            }
        })
        .collect()
}

fn compile_string<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    bytes: &'ctx [u8],
    kind: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue =
        string::compile_str_constant(context.get_llvm_module(), context.get_llvm_context(), bytes);

    memory::load_anon(context, ptr, kind)
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
