#![allow(clippy::upper_case_acronyms)]

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use crate::core::{
    compiler::backends::LinkingCompilersConfiguration,
    console::logging::{self, LoggingType},
};

pub struct GCC<'clang> {
    files: &'clang [PathBuf],
    config: &'clang LinkingCompilersConfiguration,
}

impl<'clang> GCC<'clang> {
    pub fn new(files: &'clang [PathBuf], config: &'clang LinkingCompilersConfiguration) -> Self {
        Self { files, config }
    }

    pub fn link(&self) -> Result<Duration, ()> {
        let start_time: Instant = Instant::now();

        #[cfg(target_os = "linux")]
        {
            if let Some(gcc_path) = self.config.get_custom_gcc() {
                if self.config.use_gcc() {
                    if self.handle_command(&mut self.build_gcc_command(gcc_path)) {
                        return Ok(start_time.elapsed());
                    }

                    return Err(());
                }
            }

            Err(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            logging::log(
                LoggingType::Error,
                "C compiler 'GCC' is not soported for the current operating system.",
            );

            return Err(());
        }
    }

    pub fn build_gcc_command(&self, gcc_path: &Path) -> Command {
        let mut gcc_command: Command = Command::new(gcc_path);

        gcc_command.arg("-v");

        gcc_command.args(self.config.get_args().iter());
        gcc_command.args(self.files.iter());

        if self.config.get_debug_gcc_commands() {
            logging::log(
                LoggingType::Info,
                &format!("Generated GCC Command: {:?}\n", gcc_command),
            );
        }

        gcc_command
    }

    pub fn handle_command(&self, command: &mut Command) -> bool {
        if let Ok(gcc) = command.output() {
            if !gcc.status.success() {
                if !gcc.stderr.is_empty() {
                    logging::log(
                        logging::LoggingType::Error,
                        String::from_utf8_lossy(&gcc.stderr).trim_end(),
                    );
                }

                if !gcc.stdout.is_empty() {
                    logging::log(
                        logging::LoggingType::Warning,
                        String::from_utf8_lossy(&gcc.stdout).trim_end(),
                    );
                }

                return false;
            }

            return true;
        }

        false
    }
}
