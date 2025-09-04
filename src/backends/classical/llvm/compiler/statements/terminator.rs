#![allow(clippy::collapsible_if)]

use std::fmt::Display;

use inkwell::builder::Builder;

use crate::{
    backends::classical::llvm::compiler::codegen::{self, LLVMCodegen},
    core::console::logging::{self, LoggingType},
    frontends::classical::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    if let Ast::Return {
        expression, kind, ..
    } = stmt
    {
        let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

        if expression.is_none() {
            if llvm_builder.build_return(None).is_err() {
                {
                    self::codegen_abort("Unable to build the terminator at code generation time.");
                }
            }
        }

        if let Some(expr) = expression {
            if llvm_builder
                .build_return(Some(&codegen::compile_expr(
                    codegen.get_mut_context(),
                    expr,
                    Some(kind),
                )))
                .is_err()
            {
                self::codegen_abort("Unable to build the terminator at code generation time.");
            }
        } else {
            self::codegen_abort("Unable to build the terminator at code generation time.");
        }
    } else {
        self::codegen_abort("Expected terminator to compile.");
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
