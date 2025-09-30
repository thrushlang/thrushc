#![allow(clippy::upper_case_acronyms)]

use std::path::PathBuf;

use super::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::memory::SymbolAllocated;
use crate::backends::classical::llvm::compiler::statements::lli;
use crate::backends::classical::llvm::compiler::{self, memory};
use crate::backends::classical::llvm::compiler::{abort, builtins};

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::traits::AstExtensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,

    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        // Literals
        Ast::NullPtr { .. } => context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),

        Ast::Str { bytes, .. } => {
            compiler::generation::expressions::string::compile_str_constant(context, bytes).into()
        }

        // Function
        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => compiler::generation::expressions::call::compile(context, name, args, kind, cast),

        // Function
        // Compiles a indirect function call
        Ast::Indirect {
            function,
            function_type,
            args,
            ..
        } => compiler::generation::expressions::indirect::compile(
            context,
            function,
            args,
            function_type,
            cast,
        ),

        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast),

        // Compiles a type cast operation
        Ast::As { from, cast, .. } => compiler::generation::cast::compile(context, from, cast),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref {
            value,
            kind,
            metadata,
            ..
        } => {
            let compiled_value: BasicValueEnum = self::compile(context, value, Some(kind));

            let deref_value: BasicValueEnum = if compiled_value.is_pointer_value() {
                memory::dereference(
                    context,
                    compiled_value.into_pointer_value(),
                    kind,
                    metadata.get_llvm_metadata(),
                )
            } else {
                abort::abort_codegen(
                    context,
                    "Cannot dereference non-pointer value!",
                    value.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                );
            };

            compiler::generation::cast::try_cast(context, cast, kind, deref_value)
                .unwrap_or(deref_value)
        }

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source, indexes, ..
        } => compiler::generation::pointer::property::compile(context, source, indexes),

        // Compiles a built-in function
        Ast::Builtin { builtin, .. } => builtins::compile(context, builtin, cast),

        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);
            symbol.get_ptr().into()
        }

        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => compiler::generation::expressions::inlineasm::compile(
            context,
            assembler,
            constraints,
            args,
            kind,
            attributes,
        ),

        // Compiles an indexing operation (e.g., array access)
        Ast::Index {
            source, indexes, ..
        } => compiler::generation::pointer::index::compile(context, source, indexes),

        // Low-Level Operations
        ast if ast.is_lli() => lli::compile_advanced(context, expr, cast),

        // Fallback, Unknown expressions or statements
        what => abort::abort_codegen(
            context,
            "Unknown expression or statement!",
            what.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
