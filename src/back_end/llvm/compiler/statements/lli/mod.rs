use crate::back_end::llvm::compiler::codegen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;
use std::fmt::Display;

pub mod address;
pub mod alloc;
pub mod load;
pub mod write;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &'ctx str,
    kind: &'ctx Type,
    expr: &'ctx Ast,
    span: Span,
) {
    let value: BasicValueEnum = codegen::compile(context, expr, Some(kind));
    context.new_lli(name, kind, value, span);
}

pub fn compile_advanced<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Write {
            source,
            write_type,
            write_value,
            span,
            ..
        } => self::write::compile(context, source, write_type, write_value, *span),

        Ast::Load {
            source, kind, span, ..
        } => self::load::compile(context, source, kind, *span, cast_type),

        Ast::Address {
            source, indexes, ..
        } => self::address::compile(context, source, indexes),

        Ast::Alloc {
            alloc,
            site_allocation,
            span,
            ..
        } => self::alloc::compile(context, alloc, site_allocation, *span),

        _ => {
            self::codegen_abort("Failed to compile low-level instruction. Unknown expression.");
        }
    }
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
