use std::{fmt::Display, rc::Rc};

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::ThrushType},
};

pub mod math;
pub mod mem;
pub mod sizeof;

#[derive(Debug, Clone)]
pub enum Builtin<'ctx> {
    // Memory Builtins
    AlignOf {
        align_of: ThrushType,
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

    // Math Builtins
    Sqrt {
        value: Rc<Ast<'ctx>>,
    },
    Sin {
        value: Rc<Ast<'ctx>>,
    },
    Cos {
        value: Rc<Ast<'ctx>>,
    },
    Floor {
        value: Rc<Ast<'ctx>>,
    },
    Trunc {
        value: Rc<Ast<'ctx>>,
    },
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: &'ctx Builtin<'ctx>,
    cast_type: Option<&ThrushType>,
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

        Builtin::Sqrt { value } => math::sqrt::compile(context, value),

        _ => {
            codegen_abort("Builtin not implemented.");
            compile_null_ptr(context)
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
