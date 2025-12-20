use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use inkwell::targets::TargetTriple;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::linking::LinkingCompilersConfiguration;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

#[derive(Debug)]
pub struct Clang<'clang> {
    files: &'clang [PathBuf],
    config: &'clang LinkingCompilersConfiguration,
    backend: &'clang LLVMBackend,
}

impl<'clang> Clang<'clang> {
    pub fn new(
        files: &'clang [PathBuf],
        config: &'clang LinkingCompilersConfiguration,
        backend: &'clang LLVMBackend,
    ) -> Self {
        Self {
            files,
            config,
            backend,
        }
    }
}

impl<'clang> Clang<'clang> {
    pub fn link(&self) -> Result<Duration, ()> {
        let start_time: Instant = Instant::now();

        if !self.config.get_use_clang() {
            return Err(());
        }

        let clang_path: &Path = self.config.get_custom_clang();

        let mut cmd: Command = self.build_clang_command(clang_path);

        if self.handle_command(&mut cmd) {
            return Ok(start_time.elapsed());
        }

        Ok(start_time.elapsed())
    }
}

impl Clang<'_> {
    pub fn build_clang_command(&self, clang_path: &Path) -> Command {
        let mut clang_command: Command = Command::new(clang_path);

        clang_command.arg("-v");

        let triple: &TargetTriple = self.backend.get_target().get_triple();

        clang_command.arg("-target");
        clang_command.arg(triple.as_str().to_string_lossy().into_owned());

        clang_command.args(self.files.iter());
        clang_command.args(self.config.get_args().iter());

        if self.config.get_debug_clang_commands() {
            logging::print_debug(
                LoggingType::Debug,
                &format!("Generated Clang command: '{:?}'.\n", clang_command),
            );
        }

        clang_command
    }
}

impl Clang<'_> {
    pub fn handle_command(&self, command: &mut Command) -> bool {
        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    return true;
                }

                let stderr: String = String::from_utf8_lossy(&output.stderr)
                    .trim_end()
                    .to_string();

                if !stderr.is_empty() {
                    logging::print_error(LoggingType::Error, &stderr);
                }

                let stdout: String = String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_string();

                if !stdout.is_empty() {
                    logging::print_warn(LoggingType::Warning, &stdout);
                }

                false
            }

            _ => false,
        }
    }
}
