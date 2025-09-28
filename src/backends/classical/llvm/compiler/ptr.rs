#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::builtins;
use crate::backends::classical::llvm::compiler::memory::SymbolAllocated;
use crate::backends::classical::llvm::compiler::statements::lli;
use crate::backends::classical::llvm::compiler::{self, memory};

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::traits::AstExtensions;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;

use std::fmt::Display;

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

        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => compiler::generation::pointer::call::compile(context, name, args, kind, cast),

        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast),

        // Compiles a type cast operation
        Ast::As { from, cast, .. } => compiler::generation::cast::compile(context, from, cast),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref { value, kind, .. } => {
            let val: BasicValueEnum = self::compile(context, value, Some(kind));

            let deref_value: BasicValueEnum = if val.is_pointer_value() {
                memory::load_anon(context, val.into_pointer_value(), kind)
            } else {
                self::codegen_abort(format!(
                    "Cannot dereference non-pointer value in '{}'.",
                    value
                ));
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
        } => compiler::generation::pointer::inlineasm::compile(
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
        what => {
            self::codegen_abort(format!(
                "Failed to compile. Unknown expression or statement '{:?}'.",
                what
            ));
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message))
}
