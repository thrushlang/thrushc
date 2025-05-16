use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use crate::{
    EXECUTABLE_EXTENSION, LLVM_BACKEND,
    backend::thrushc::{self},
    standard::logging,
};

pub struct LLVMLinker<'lld> {
    files: &'lld [PathBuf],
    flags: &'lld str,
}

impl<'lld> LLVMLinker<'lld> {
    pub fn new(files: &'lld [PathBuf], flags: &'lld str) -> Self {
        Self { files, flags }
    }

    pub fn link(&self) -> Duration {
        let lld_time: Instant = Instant::now();

        let system_executables_extension: &str = &EXECUTABLE_EXTENSION;

        let lld_path: PathBuf =
            LLVM_BACKEND.join(format!("ld.lld{}", system_executables_extension));

        if !lld_path.exists() {
            logging::log(
                logging::LoggingType::Panic,
                &format!(
                    "Missing linker of the LLVM Toolchain: 'ld.lld{}'. Maybe it's time to use 'thorium toolchain llvm repair'.",
                    system_executables_extension
                ),
            )
        }

        let flags: Vec<&str> = self.create_flags();

        let mut lld: Command = Command::new(lld_path);

        lld.args(self.files);
        lld.args(flags);

        thrushc::handler::handle_command(&mut lld);

        lld_time.elapsed()
    }

    fn create_flags(&self) -> Vec<&str> {
        self.flags.split(";").collect()
    }
}
