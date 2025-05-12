use std::process::Command;

use crate::LLVM_BACKEND;

use super::handler;

pub struct LLVMOptimizer;

impl LLVMOptimizer {
    pub fn optimize(path: &str, opt: &str) {
        handler::handle_command(
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("tools/opt"))
                .arg(format!("-p={}", opt))
                .arg(path)
                .arg("-o")
                .arg(path),
        );
    }
}
