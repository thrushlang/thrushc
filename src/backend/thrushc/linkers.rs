use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use crate::{LLVM_BACKEND, standard::misc::CompilerOptions};

use super::handler;

pub struct LLVMLinker<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

impl<'a> LLVMLinker<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn link(&self) -> Duration {
        let start_time: Instant = Instant::now();

        let mut llvm_link_command: Command =
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("ld.lld"));

        llvm_link_command.args(
            self.options
                .get_llvm_backend_options()
                .get_linker_arguments(),
        );

        llvm_link_command.args(self.files);

        handler::handle_command(&mut llvm_link_command);

        start_time.elapsed()
    }
}
