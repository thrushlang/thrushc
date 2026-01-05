use inkwell::basic_block::BasicBlock;

#[derive(Debug)]
pub struct LLVMLoopContext<'ctx> {
    break_branches: Vec<BasicBlock<'ctx>>,
    continue_branches: Vec<BasicBlock<'ctx>>,
}

impl<'ctx> LLVMLoopContext<'ctx> {
    #[inline]
    pub fn new() -> LLVMLoopContext<'ctx> {
        LLVMLoopContext {
            break_branches: Vec::with_capacity(255),
            continue_branches: Vec::with_capacity(255),
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
            self::codegen_abort("loop control flow 'breaker' branch couldn't be obtained.");
        })
    }

    #[inline]
    pub fn get_last_continue_branch(&self) -> BasicBlock<'ctx> {
        *self.continue_branches.last().unwrap_or_else(|| {
            self::codegen_abort("loop control flow 'continue' branch couldn't be obtained.");
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

fn codegen_abort<T: std::fmt::Display>(message: T) -> ! {
    thrushc_logging::print_backend_bug(
        thrushc_logging::LoggingType::BackendBug,
        &format!("{}", message),
    );
}
