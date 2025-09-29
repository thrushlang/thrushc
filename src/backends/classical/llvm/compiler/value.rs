#![allow(clippy::upper_case_acronyms)]

use std::path::PathBuf;

use super::context::LLVMCodeGenContext;

use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::abort;
use crate::backends::classical::llvm::compiler::binaryop;
use crate::backends::classical::llvm::compiler::builtins;
use crate::backends::classical::llvm::compiler::generation::{float, int};
use crate::backends::classical::llvm::compiler::memory::{self};
use crate::backends::classical::llvm::compiler::statements::lli;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::traits::AstExtensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,

    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        // Literal Expressions
        Ast::Float {
            kind,
            value,
            signed,
            ..
        } => {
            let float: BasicValueEnum =
                float::generate(context.get_llvm_context(), kind, *value, *signed).into();

            compiler::generation::cast::try_cast(context, cast, kind, float).unwrap_or(float)
        }

        Ast::Integer {
            kind,
            value,
            signed,
            ..
        } => {
            let int: BasicValueEnum =
                int::generate(context.get_llvm_context(), kind, *value, *signed).into();

            compiler::generation::cast::try_cast(context, cast, kind, int).unwrap_or(int)
        }

        Ast::NullPtr { .. } => context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),

        Ast::Str { bytes, kind, .. } => {
            let ptr: PointerValue =
                compiler::generation::expressions::string::compile_str_constant(context, bytes);

            memory::load_anon(context, ptr, kind)
        }

        Ast::Char { byte, .. } => context
            .get_llvm_context()
            .i8_type()
            .const_int(*byte, false)
            .into(),

        Ast::Boolean { value, .. } => context
            .get_llvm_context()
            .bool_type()
            .const_int(*value, false)
            .into(),

        // Function and Built-in Calls
        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => compiler::generation::value::call::compile(context, name, args, kind, cast),

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
            span,
            ..
        } => match binaryop_type {
            t if t.is_float_type() => {
                binaryop::float::compile(context, (left, operator, right, *span), cast)
            }
            t if t.is_integer_type() => {
                binaryop::integer::compile(context, (left, operator, right, *span), cast)
            }
            t if t.is_bool_type() => {
                binaryop::boolean::compile(context, (left, operator, right, *span), cast)
            }
            t if t.is_ptr_type() => {
                binaryop::pointer::compile(context, (left, operator, right, *span))
            }

            _ => {
                abort::abort_codegen(
                    context,
                    "Can't be compiled!.",
                    *span,
                    PathBuf::from(file!()),
                    line!(),
                );
            }
        },

        Ast::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } => compiler::generation::expressions::unary::compile(
            context,
            (operator, kind, expression),
            cast,
        ),

        // Symbol/Property Access
        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).load(context),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source, indexes, ..
        } => compiler::generation::expressions::property::compile(context, source, indexes),

        // Memory Access Operations
        // Compiles an indexing operation (e.g., array access)
        Ast::Index {
            source, indexes, ..
        } => compiler::generation::value::index::compile(context, source, indexes),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref {
            value,
            kind,
            metadata,
            ..
        } => {
            let value: BasicValueEnum = self::compile(context, value, Some(kind));

            let deref_value: BasicValueEnum = if value.is_pointer_value() {
                memory::dereference(
                    context,
                    value.into_pointer_value(),
                    kind,
                    metadata.get_llvm_metadata(),
                )
            } else {
                value
            };

            compiler::generation::cast::try_cast(context, cast, kind, deref_value)
                .unwrap_or(deref_value)
        }

        // Array Operations
        // Compiles a fixed-size array
        Ast::FixedArray { items, kind, .. } => {
            compiler::generation::expressions::farray::compile(context, items, kind, cast)
        }

        // Compiles a dynamic array
        Ast::Array { items, kind, .. } => {
            compiler::generation::expressions::array::compile(context, items, kind, cast)
        }

        // Compiles a struct constructor
        Ast::Constructor { args, kind, .. } => {
            compiler::generation::structgen::compile(context, args, kind, cast)
        }

        // Compiles a type cast operation
        Ast::As { from, cast, .. } => compiler::generation::cast::compile(context, from, cast),

        // Low-Level Operations
        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => compiler::generation::value::inlineasm::compile(
            context,
            assembler,
            constraints,
            args,
            kind,
            attributes,
        ),

        // Low-Level Operations
        ast if ast.is_lli() => lli::compile_advanced(context, expr, cast),

        // Fallback, Unknown expressions or statements
        what => {
            abort::abort_codegen(
                context,
                "Unknown expression or statement!",
                what.get_span(),
                PathBuf::from(file!()),
                line!(),
            );
        }
    }
}
