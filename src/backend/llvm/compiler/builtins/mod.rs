use std::{fmt::Display, rc::Rc};

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
    frontend::types::parser::stmts::stmt::ThrushStatement,
};

pub mod math;
pub mod mem;
pub mod sizeof;

#[derive(Debug, Clone)]
pub enum Builtin<'ctx> {
    // Memory Builtins
    MemCpy {
        source: Rc<ThrushStatement<'ctx>>,
        destination: Rc<ThrushStatement<'ctx>>,
        size: Rc<ThrushStatement<'ctx>>,
    },
    MemMove {
        source: Rc<ThrushStatement<'ctx>>,
        destination: Rc<ThrushStatement<'ctx>>,
        size: Rc<ThrushStatement<'ctx>>,
    },
    MemSet {
        destination: Rc<ThrushStatement<'ctx>>,
        new_size: Rc<ThrushStatement<'ctx>>,
        size: Rc<ThrushStatement<'ctx>>,
    },

    // Math Builtins
    Sqrt {
        value: Rc<ThrushStatement<'ctx>>,
    },
    Sin {
        value: Rc<ThrushStatement<'ctx>>,
    },
    Cos {
        value: Rc<ThrushStatement<'ctx>>,
    },
    Floor {
        value: Rc<ThrushStatement<'ctx>>,
    },
    Trunc {
        value: Rc<ThrushStatement<'ctx>>,
    },
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: &'ctx Builtin<'ctx>,
) -> BasicValueEnum<'ctx> {
    match builtin {
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
    logging::log(
        LoggingType::Bug,
        &format!("CODE GENERATION: '{}'.", message),
    );
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
