use colored::Colorize;

use inkwell::targets::TargetTriple;
use thrushc_options::{backends::llvm::LLVMBackend, linkage::LinkingCompilersConfiguration};

use crate::ThrushCompiler;

#[derive(Debug)]
pub struct ClangLinker<'clang> {
    files: &'clang [std::path::PathBuf],
    config: &'clang LinkingCompilersConfiguration,
    backend: &'clang LLVMBackend,
}

impl<'clang> ClangLinker<'clang> {
    pub fn new(
        files: &'clang [std::path::PathBuf],
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

impl<'clang> ClangLinker<'clang> {
    pub fn link(&self) -> Result<std::time::Duration, ()> {
        let start_time: std::time::Instant = std::time::Instant::now();

        if !self.config.get_use_clang() {
            return Err(());
        }

        let clang_path: &std::path::Path = self.config.get_custom_clang();

        let mut cmd: std::process::Command = self.build_clang_command(clang_path);

        if self.handle_command(&mut cmd) {
            return Ok(start_time.elapsed());
        }

        Ok(start_time.elapsed())
    }
}

impl ClangLinker<'_> {
    pub fn build_clang_command(&self, clang_path: &std::path::Path) -> std::process::Command {
        let mut clang_command: std::process::Command = std::process::Command::new(clang_path);

        clang_command.arg("-v");

        let triple: &TargetTriple = self.backend.get_target().get_triple();

        clang_command.arg("-target");
        clang_command.arg(triple.as_str().to_string_lossy().into_owned());

        clang_command.args(self.files.iter());
        clang_command.args(self.config.get_args().iter());

        if self.config.get_debug_clang_commands() {
            thrushc_logging::print_debug(
                thrushc_logging::LoggingType::Debug,
                &format!("Generated Clang command: '{:?}'.\n", clang_command),
            );
        }

        clang_command
    }
}

impl ClangLinker<'_> {
    fn handle_command(&self, command: &mut std::process::Command) -> bool {
        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    return true;
                }

                let stderr: String = String::from_utf8_lossy(&output.stderr)
                    .trim_end()
                    .to_string();

                if !stderr.is_empty() {
                    thrushc_logging::print_error(thrushc_logging::LoggingType::Error, &stderr);
                }

                let stdout: String = String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_string();

                if !stdout.is_empty() {
                    thrushc_logging::print_warn(thrushc_logging::LoggingType::Warning, &stdout);
                }

                false
            }

            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct GCCLinker<'gcc> {
    files: &'gcc [std::path::PathBuf],
    config: &'gcc LinkingCompilersConfiguration,
}

impl<'gcc> GCCLinker<'gcc> {
    #[inline]
    pub fn new(
        files: &'gcc [std::path::PathBuf],
        config: &'gcc LinkingCompilersConfiguration,
    ) -> Self {
        Self { files, config }
    }
}

impl<'gcc> GCCLinker<'gcc> {
    pub fn link(&self) -> Result<std::time::Duration, ()> {
        let start_time: std::time::Instant = std::time::Instant::now();

        if !self.config.get_use_gcc() {
            return Err(());
        }

        let gcc_path: &std::path::Path = self.config.get_custom_gcc();

        let mut cmd: std::process::Command = self.build_gcc_command(gcc_path);

        if self.handle_command(&mut cmd) {
            return Ok(start_time.elapsed());
        }

        Ok(start_time.elapsed())
    }
}

impl GCCLinker<'_> {
    pub fn build_gcc_command(&self, gcc_path: &std::path::Path) -> std::process::Command {
        let mut gcc_command: std::process::Command = std::process::Command::new(gcc_path);

        gcc_command.arg("-v");
        gcc_command.args(self.files.iter());
        gcc_command.args(self.config.get_args().iter());

        if self.config.get_debug_gcc_commands() {
            thrushc_logging::print_debug(
                thrushc_logging::LoggingType::Debug,
                &format!("Generated GCC command: {:?}\n", gcc_command),
            );
        }

        gcc_command
    }
}

impl GCCLinker<'_> {
    pub fn handle_command(&self, command: &mut std::process::Command) -> bool {
        match command.output() {
            Ok(output) if output.status.success() => true,

            Ok(output) => {
                if !output.stderr.is_empty() {
                    thrushc_logging::print_error(
                        thrushc_logging::LoggingType::Error,
                        String::from_utf8_lossy(&output.stderr).trim_end(),
                    );
                }

                if !output.stdout.is_empty() {
                    thrushc_logging::print_warn(
                        thrushc_logging::LoggingType::Warning,
                        String::from_utf8_lossy(&output.stdout).trim_end(),
                    );
                }

                false
            }

            _ => false,
        }
    }
}

pub fn link_with_clang(compiler: &mut ThrushCompiler) {
    let llvm_backend: &LLVMBackend = compiler.get_options().get_llvm_backend_options();

    let linking_compiler_config: &LinkingCompilersConfiguration =
        compiler.get_options().get_linking_compilers_configuration();

    let all_compiled_files: &[std::path::PathBuf] = compiler.get_compiled_files();

    if let Ok(clang_time) =
        ClangLinker::new(all_compiled_files, linking_compiler_config, llvm_backend).link()
    {
        compiler.linking_time += clang_time;

        thrushc_logging::write(
            thrushc_logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold()
            ),
        );
    } else {
        thrushc_logging::write(
            thrushc_logging::OutputIn::Stderr,
            &format!(
                "\r{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FAILED".bright_red().bold()
            ),
        );
    }
}

pub fn link_with_gcc(compiler: &mut ThrushCompiler) {
    let linking_compiler_configuration: &LinkingCompilersConfiguration =
        compiler.get_options().get_linking_compilers_configuration();

    let all_compiled_files: &[std::path::PathBuf] = compiler.get_compiled_files();

    if let Ok(gcc_time) = GCCLinker::new(all_compiled_files, linking_compiler_configuration).link()
    {
        compiler.linking_time += gcc_time;

        thrushc_logging::write(
            thrushc_logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold()
            ),
        );
    } else {
        thrushc_logging::write(
            thrushc_logging::OutputIn::Stderr,
            &format!(
                "\r{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FAILED".bright_red().bold()
            ),
        );
    }
}
