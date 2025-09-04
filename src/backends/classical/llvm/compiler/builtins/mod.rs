use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use std::rc::Rc;

use inkwell::values::BasicValueEnum;

pub mod mem;
pub mod sizeof;

#[derive(Debug, Clone)]
pub enum Builtin<'ctx> {
    // Memory Builtins
    Halloc {
        alloc: Type,
    },
    MemCpy {
        source: Rc<Ast<'ctx>>,
        destination: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
    },
    MemMove {
        source: Rc<Ast<'ctx>>,
        destination: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
    },
    MemSet {
        destination: Rc<Ast<'ctx>>,
        new_size: Rc<Ast<'ctx>>,
        size: Rc<Ast<'ctx>>,
    },
    AlignOf {
        align_of: Type,
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
        } => mem::memcpy::compile(context, source, destination, size),

        Builtin::MemMove {
            source,
            destination,
            size,
        } => mem::memmove::compile(context, source, destination, size),

        Builtin::MemSet {
            destination,
            new_size,
            size,
        } => mem::memset::compile(context, destination, new_size, size),

        Builtin::Halloc { alloc } => mem::halloc::compile(context, alloc),
    }
}
