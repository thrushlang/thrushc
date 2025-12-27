use crate::back_end::llvm_codegen::constants::LLVM_COMPILER_BRANCHERS_CAPACITY;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use inkwell::basic_block::BasicBlock;
use std::fmt::Display;

#[derive(Debug)]
pub struct LLVMLoopContext<'ctx> {
    break_branches: Vec<BasicBlock<'ctx>>,
    continue_branches: Vec<BasicBlock<'ctx>>,
}

impl<'ctx> LLVMLoopContext<'ctx> {
    #[inline]
    pub fn new() -> LLVMLoopContext<'ctx> {
        LLVMLoopContext {
            break_branches: Vec::with_capacity(LLVM_COMPILER_BRANCHERS_CAPACITY),
            continue_branches: Vec::with_capacity(LLVM_COMPILER_BRANCHERS_CAPACITY),
        }
    }
}

impl<'ctx> LLVMLoopContext<'ctx> {
    #[inline]
    pub fn add_break_branch(&mut self, branch: BasicBlock<'ctx>) {
        self.break_branches.push(branch);
    }

    #[inline]
    pub fn add_continue_branch(&mut self, branch: BasicBlock<'ctx>) {
        self.continue_branches.push(branch);
    }
}

impl<'ctx> LLVMLoopContext<'ctx> {
    #[inline]
    pub fn get_last_break_branch(&self) -> BasicBlock<'ctx> {
        *self.break_branches.last().unwrap_or_else(|| {
            self::codegen_abort("Break point branch couldn't be obtained.");
        })
    }

    #[inline]
    pub fn get_last_continue_branch(&self) -> BasicBlock<'ctx> {
        *self.continue_branches.last().unwrap_or_else(|| {
            self::codegen_abort("Continue point branch couldn't be obtained.");
        })
    }
}

impl LLVMLoopContext<'_> {
    #[inline]
    pub fn pop(&mut self) {
        self.break_branches.pop();
        self.continue_branches.pop();
    }
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
