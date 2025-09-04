#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use super::typegen;

use crate::backends::classical::llvm::compiler::attributes::LLVMAttribute;
use crate::backends::classical::llvm::compiler::builtins;
use crate::backends::classical::llvm::compiler::cast;
use crate::backends::classical::llvm::compiler::codegen;
use crate::backends::classical::llvm::compiler::expressions;
use crate::backends::classical::llvm::compiler::indexes;
use crate::backends::classical::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backends::classical::llvm::compiler::ptrgen;
use crate::backends::classical::llvm::compiler::statements::lli;
use crate::backends::classical::llvm::compiler::valuegen;

use crate::backends::classical::types::LLVMEitherExpression;
use crate::backends::classical::types::repr::LLVMFunction;
use crate::backends::classical::types::traits::AssemblerFunctionExtensions;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::traits::AstExtensions;
use crate::frontends::classical::types::ast::types::AstEitherExpression;
use crate::frontends::classical::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::types::{BasicTypeEnum, FunctionType, PointerType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue, StructValue,
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
        // Literals
        Ast::NullPtr { .. } => self::compile_null_ptr(context),
        Ast::Str { bytes, .. } => expressions::string::compile_str_constant(context, bytes).into(),

        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => self::compile_function_call(context, name, args, kind, cast),

        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast),

        // Compiles a type cast operation
        Ast::As { from, cast, .. } => self::compile_cast(context, from, cast),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref { value, kind, .. } => self::compile_deref(context, value, kind, cast),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source, indexes, ..
        } => self::compile_property(context, source, indexes),

        // Compiles a built-in function
        Ast::Builtin { builtin, .. } => builtins::compile(context, builtin, cast),

        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => self::compile_reference(context, name),

        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => self::compile_inline_asm(context, assembler, constraints, args, kind, attributes),

        // Compiles an indexing operation (e.g., array access)
        Ast::Index {
            source, indexes, ..
        } => self::compile_index(context, source, indexes),

        // Low-Level Operations
        ast if ast.is_lli() => lli::compile_advanced(context, expr, cast),

        // Fallback, Unknown expressions or statements
        what => {
            self::codegen_abort(format!(
                "Failed to compile. Unknown expression or statement '{:?}'.",
                what
            ));
        }
    }
}

fn compile_function_call<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    args: &'ctx [Ast],
    kind: &'ctx Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let function: LLVMFunction = context.get_table().get_function(name);

    let (llvm_function, function_arg_types, function_convention) =
        (function.0, function.1, function.2);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .enumerate()
        .map(|(idx, expr)| {
            let cast: Option<&Type> = function_arg_types.get(idx);

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
                })
            } else {
                self::compile_null_ptr(context)
            }
        }
        Err(_) => {
            self::codegen_abort(format!("Failed to generate call to function '{}'.", name));
        }
    };

    cast::try_cast(context, cast, kind, fn_value).unwrap_or(fn_value)
}

fn compile_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let from_type: &Type = from.get_type_unwrapped();

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let abort_value =
        |_| self::codegen_abort(format!("Failed to cast '{}' to '{}'.", from_type, cast));

    let abort_ptr =
        |_| self::codegen_abort(format!("Failed to cast '{}' to '{}'.", from_type, cast));

    match (from_type, cast) {
        (from_type, cast) if from_type.is_str_type() && cast.is_ptr_type() => {
            let val: BasicValueEnum = ptrgen::compile(context, from, Some(cast));

            let str_structure: StructValue = if val.is_pointer_value() {
                let raw_str_ptr: PointerValue = val.into_pointer_value();
                memory::load_anon(context, raw_str_ptr, from_type).into_struct_value()
            } else {
                val.into_struct_value()
            };

            llvm_builder
                .build_extract_value(str_structure, 0, "")
                .unwrap_or_else(abort_value)
        }

        (from_type, cast) if cast.is_ptr_type() || cast.is_mut_type() => {
            let val: BasicValueEnum = ptrgen::compile(context, from, Some(cast));

            if val.is_pointer_value() {
                let to: PointerType =
                    typegen::generate_type(llvm_context, cast).into_pointer_type();

                llvm_builder
                    .build_pointer_cast(val.into_pointer_value(), to, "")
                    .unwrap_or_else(abort_ptr)
                    .into()
            } else {
                self::codegen_abort(format!(
                    "Expected pointer value for cast from '{}' to '{}'.",
                    from_type, cast
                ));
            }
        }

        (from_type, cast) => {
            self::codegen_abort(format!(
                "Unsupported cast from '{}' to '{}'.",
                from_type, cast
            ));
        }
    }
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
        self::codegen_abort(format!(
            "Cannot dereference non-pointer value in '{}'.",
            value
        ));
    };

    cast::try_cast(context, cast, kind, deref_value).unwrap_or(deref_value)
}

fn compile_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx AstEitherExpression<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            if !symbol.is_pointer() {
                self::codegen_abort(format!(
                    "Symbol '{}' is not a pointer for property access.",
                    name
                ));
            }

            let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);

            for index in indexes.iter().skip(1) {
                let index_type: BasicTypeEnum = typegen::generate_type(llvm_context, &index.0);

                match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
                    Ok(new_ptr) => ptr = new_ptr,
                    Err(_) => {
                        self::codegen_abort(format!(
                            "Failed to access property at index '{}' for '{}'.",
                            index.1, name
                        ));
                    }
                }
            }

            ptr.into()
        }

        (_, Some(expr)) => {
            let kind: &Type = expr.get_type_unwrapped();
            let ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();

            let mut ptr: PointerValue = memory::get_struct_anon(context, ptr, kind, indexes[0].1);

            for index in indexes.iter().skip(1) {
                let index_type: BasicTypeEnum = typegen::generate_type(llvm_context, &index.0);

                match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
                    Ok(new_ptr) => ptr = new_ptr,
                    Err(_) => {
                        self::codegen_abort(format!(
                            "Failed to access property at index '{}' for a expression.",
                            index.1
                        ));
                    }
                }
            }

            ptr.into()
        }

        _ => {
            self::codegen_abort("Unable to compile property access.");
        }
    }
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    let symbol: SymbolAllocated = context.get_table().get_symbol(name);
    symbol.get_ptr().into()
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
        .map(|arg| valuegen::compile(context, arg, None).into()) // Recursive
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
        }),

        Ok(_) => self::compile_null_ptr(context),

        Err(_) => {
            self::codegen_abort("Failed to build inline assembler.");
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
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message))
}
