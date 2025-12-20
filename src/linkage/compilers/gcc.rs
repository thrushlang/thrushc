#![allow(clippy::upper_case_acronyms)]

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::core::compiler::linking::LinkingCompilersConfiguration;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

#[derive(Debug)]
pub struct GCC<'gcc> {
    files: &'gcc [PathBuf],
    config: &'gcc LinkingCompilersConfiguration,
}

impl<'gcc> GCC<'gcc> {
    #[inline]
    pub fn new(files: &'gcc [PathBuf], config: &'gcc LinkingCompilersConfiguration) -> Self {
        Self { files, config }
    }
}

impl<'gcc> GCC<'gcc> {
    pub fn link(&self) -> Result<Duration, ()> {
        let start_time: Instant = Instant::now();

        if !self.config.get_use_gcc() {
            return Err(());
        }

        let gcc_path: &Path = self.config.get_custom_gcc();

        let mut cmd: Command = self.build_gcc_command(gcc_path);

        if self.handle_command(&mut cmd) {
            return Ok(start_time.elapsed());
        }

        Ok(start_time.elapsed())
    }
}

impl GCC<'_> {
    pub fn build_gcc_command(&self, gcc_path: &Path) -> Command {
        let mut gcc_command: Command = Command::new(gcc_path);

        gcc_command.arg("-v");

        gcc_command.args(self.files.iter());
        gcc_command.args(self.config.get_args().iter());

        if self.config.get_debug_gcc_commands() {
            logging::print_debug(
                LoggingType::Debug,
                &format!("Generated GCC command: {:?}\n", gcc_command),
            );
        }

        gcc_command
    }
}

impl GCC<'_> {
    pub fn handle_command(&self, command: &mut Command) -> bool {
        match command.output() {
            Ok(output) if output.status.success() => true,

            Ok(output) => {
                if !output.stderr.is_empty() {
                    logging::print_error(
                        LoggingType::Error,
                        String::from_utf8_lossy(&output.stderr).trim_end(),
                    );
                }

                if !output.stdout.is_empty() {
                    logging::print_warn(
                        LoggingType::Warning,
                        String::from_utf8_lossy(&output.stdout).trim_end(),
                    );
                }

                false
            }

            _ => false,
        }
    }
}
