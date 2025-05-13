use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use crate::{LLVM_BACKEND, backend::thrushc::handler, standard::logging};

pub struct Clang<'clang> {
    files: &'clang [PathBuf],
    arguments: &'clang [String],
}

impl<'clang> Clang<'clang> {
    pub fn new(files: &'clang [PathBuf], arguments: &'clang [String]) -> Self {
        Self { files, arguments }
    }

    pub fn compile(&self) -> Duration {
        let clang_time: Instant = Instant::now();

        let clang_path: PathBuf = LLVM_BACKEND.join("clang");

        if !clang_path.exists() {
            logging::log(
                logging::LoggingType::Panic,
                "Missing compiler of LLVM Toolchain: 'clang'. Maybe it's time to use 'thorium toolchain llvm repair'.",
            )
        }

        let mut clang: Command = Command::new(clang_path);

        clang.args(self.arguments);
        clang.args(self.files);

        handler::handle_command(&mut clang);

        clang_time.elapsed()
    }
}
