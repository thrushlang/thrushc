use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use crate::{LLVM_BACKEND, standard::misc::CompilerOptions};

use super::handler;

pub struct LLVMStaticCompiler<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

impl<'a> LLVMStaticCompiler<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) -> Duration {
        let start_time: Instant = Instant::now();

        let mut llvm_link_command: Command =
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("llc"));

        llvm_link_command.args(
            self.options
                .get_llvm_backend_options()
                .get_static_compiler_arguments(),
        );

        llvm_link_command.args(self.files);

        handler::handle_command(&mut llvm_link_command);

        start_time.elapsed()
    }
}
