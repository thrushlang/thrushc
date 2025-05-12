use std::{path::PathBuf, process::Command};

use crate::LLVM_BACKEND;

use super::handler;

pub struct LLVMDisassembler<'a> {
    files: &'a [PathBuf],
}

impl<'a> LLVMDisassembler<'a> {
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }

    pub fn dissamble(&self) {
        handler::handle_command(
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("tools/llvm-dis")).args(self.files),
        );
    }
}
