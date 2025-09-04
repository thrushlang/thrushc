use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use inkwell::targets::TargetTriple;

use crate::core::{
    compiler::{backends::llvm::LLVMBackend, linking::LinkingCompilersConfiguration},
    console::logging::{self, LoggingType},
};

use super::decompressor;

#[cfg(target_os = "linux")]
pub static LINUX_X86_64_CLANG: &[u8] =
    include_bytes!("../../../../../embedded/compilers/linux/clang/clang-linux-x86_64.tar.xz");

#[cfg(target_os = "linux")]
pub static LINUX_X86_64_CLANG_MANIFEST: &str =
    include_str!("../../../../../embedded/compilers/linux/clang/clang-manifest.json");

#[cfg(target_os = "windows")]
pub static WINDOWS_X86_64_CLANG: &[u8] =
    include_bytes!("../../../../embedded/compilers/windows/clang/clang-windows-x86_64.zip");

#[cfg(target_os = "windows")]
pub static WINDOWS_X86_64_CLANG_MANIFEST: &str =
    include_str!("../../../../embedded/compilers/windows/clang/clang-manifest.json");

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

    pub fn link(&self) -> Result<Duration, ()> {
        let start_time: Instant = Instant::now();

        #[cfg(target_os = "linux")]
        {
            if self.config.get_use_clang() {
                if let Some(custom_clang) = self.config.get_custom_clang() {
                    if self.handle_command(&mut self.build_clang_command(custom_clang)) {
                        return Ok(start_time.elapsed());
                    }

                    return Err(());
                }

                let embedded_raw_clang: (&'static [u8], &'static str, PathBuf, PathBuf, PathBuf) =
                    get_x86_64_linux_clang();
                let clang_raw_bytes: &'static [u8] = embedded_raw_clang.0;
                let clang_manifest: &'static str = embedded_raw_clang.1;
                let clang_manifest_path: PathBuf = embedded_raw_clang.2;
                let clang_tar_path: PathBuf = embedded_raw_clang.3;
                let clang_output_path: PathBuf = embedded_raw_clang.4;

                if let Ok(clang_path) = decompressor::dump_x86_64_clang_linux(
                    clang_manifest,
                    clang_raw_bytes,
                    clang_manifest_path,
                    clang_tar_path,
                    clang_output_path,
                ) {
                    if self.handle_command(&mut self.build_clang_command(&clang_path)) {
                        return Ok(start_time.elapsed());
                    }

                    return Err(());
                }

                return Err(());
            }

            Err(())
        }

        #[cfg(target_os = "windows")]
        {
            if self.config.use_clang() {
                if let Some(custom_clang) = self.config.get_custom_clang() {
                    if self.handle_command(&mut self.build_clang_command(custom_clang)) {
                        return Ok(start_time.elapsed());
                    }
                    return Err(());
                }

                let embedded_raw_clang: (&'static [u8], &'static str, PathBuf, PathBuf, PathBuf) =
                    get_x86_64_windows_clang();
                let clang_raw_bytes: &'static [u8] = embedded_raw_clang.0;
                let clang_manifest: &'static str = embedded_raw_clang.1;
                let clang_manifest_path: PathBuf = embedded_raw_clang.2;
                let clang_tar_path: PathBuf = embedded_raw_clang.3;
                let clang_output_path: PathBuf = embedded_raw_clang.4;

                if let Ok(clang_path) = decompressor::dump_x86_64_clang_windows(
                    clang_manifest,
                    clang_raw_bytes,
                    clang_manifest_path,
                    clang_tar_path,
                    clang_output_path,
                ) {
                    if self.handle_command(&mut self.build_clang_command(&clang_path)) {
                        return Ok(start_time.elapsed());
                    }

                    return Err(());
                }

                return Err(());
            }

            return Err(());
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            logging::log(
                LoggingType::Error,
                "C compiler 'clang' is not supported for the current operating system.",
            );

            Err(())
        }
    }

    pub fn build_clang_command(&self, clang_path: &Path) -> Command {
        let mut clang_command: Command = Command::new(clang_path);

        clang_command.arg("-v");

        let triple: &TargetTriple = self.backend.get_target().get_triple();

        clang_command.arg("-target");
        clang_command.arg(triple.as_str().to_string_lossy().into_owned());

        clang_command.args(self.files.iter());
        clang_command.args(self.config.get_args().iter());

        if self.config.get_debug_clang_commands() {
            logging::log(
                LoggingType::Info,
                &format!("Generated Clang Command: '{:?}'.\n", clang_command),
            );
        }

        clang_command
    }

    pub fn handle_command(&self, command: &mut Command) -> bool {
        if let Ok(clang) = command.output() {
            if !clang.status.success() {
                if !clang.stderr.is_empty() {
                    logging::log(
                        logging::LoggingType::Error,
                        String::from_utf8_lossy(&clang.stderr).trim_end(),
                    );
                }

                if !clang.stdout.is_empty() {
                    logging::log(
                        logging::LoggingType::Warning,
                        String::from_utf8_lossy(&clang.stdout).trim_end(),
                    );
                }

                return false;
            }

            return true;
        }

        false
    }
}

#[cfg(target_os = "linux")]
pub fn get_x86_64_linux_clang() -> (&'static [u8], &'static str, PathBuf, PathBuf, PathBuf) {
    (
        LINUX_X86_64_CLANG,
        LINUX_X86_64_CLANG_MANIFEST,
        PathBuf::from("clang-manifest.json"),
        PathBuf::from("clang-linux-x86_64.tar.xz"),
        PathBuf::from("clang-17"),
    )
}

#[cfg(target_os = "windows")]
pub fn get_x86_64_windows_clang() -> (&'static [u8], &'static str, PathBuf, PathBuf, PathBuf) {
    (
        WINDOWS_X86_64_CLANG,
        WINDOWS_X86_64_CLANG_MANIFEST,
        PathBuf::from("clang-manifest.json"),
        PathBuf::from("clang-windows-x86_64.zip"),
        PathBuf::from("bin"),
    )
}
