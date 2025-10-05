use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use std::rc::Rc;

use inkwell::values::BasicValueEnum;

pub mod mem;

#[derive(Debug, Clone)]
pub enum Builtin<'ctx> {
    // Memory Builtins
    Halloc {
        alloc: Type,
        span: Span,
    },
    MemCpy {
        source: Rc<Ast<'ctx>>,
        destination: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
        span: Span,
    },
    MemMove {
        source: Rc<Ast<'ctx>>,
        destination: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
        span: Span,
    },
    MemSet {
        destination: Rc<Ast<'ctx>>,
        new_size: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
        span: Span,
    },
    AlignOf {
        align_of: Type,
    },
    SizeOf {
        size_of: Type,
        span: Span,
    },
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: &'ctx Builtin<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match builtin {
        Builtin::AlignOf { align_of } => mem::alingof::compile(context, align_of, cast_type),
        Builtin::MemCpy {
            source,
            destination,
            size,
            span,
        } => mem::memcpy::compile(context, source, destination, size, *span),
        Builtin::MemMove {
            source,
            destination,
            size,
            span,
        } => mem::memmove::compile(context, source, destination, size, *span),
        Builtin::MemSet {
            destination,
            new_size,
            size,
            span,
        } => mem::memset::compile(context, destination, new_size, size, *span),
        Builtin::Halloc { alloc, span } => mem::halloc::compile(context, alloc, *span),
        Builtin::SizeOf { size_of, span } => {
            mem::sizeof::compile(context, size_of, cast_type, *span)
        }
    }
}
