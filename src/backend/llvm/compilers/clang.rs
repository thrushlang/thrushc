use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use crate::standard::{backends::CompilersConfiguration, logging};

use super::decompressor;

pub static LINUX_X86_64_CLANG: &[u8] =
    include_bytes!("../../../../embedded/compilers/clang.tar.xz");

pub struct Clang<'clang> {
    files: &'clang [PathBuf],
    build_dir: &'clang PathBuf,
    config: &'clang CompilersConfiguration,
}

impl<'clang> Clang<'clang> {
    pub fn new(
        files: &'clang [PathBuf],
        build_dir: &'clang PathBuf,
        config: &'clang CompilersConfiguration,
    ) -> Self {
        Self {
            files,
            build_dir,
            config,
        }
    }

    pub fn link(&self) -> Result<Duration, ()> {
        let start_time: Instant = Instant::now();

        if cfg!(target_os = "linux") {
            if self.config.use_clang() {
                let embedded_raw_clang: (&'static [u8], PathBuf, PathBuf) =
                    self::get_x86_64_linux_clang();

                let raw_bytes: &'static [u8] = embedded_raw_clang.0;
                let tar_path: PathBuf = embedded_raw_clang.1;
                let output_path: PathBuf = embedded_raw_clang.2;

                if let Ok(clang_path) = decompressor::dump_x86_64_linux_clang(
                    self.build_dir,
                    raw_bytes,
                    tar_path,
                    output_path,
                ) {
                    if self.handle_command(&mut self.build_clang_command(&clang_path)) {
                        return Ok(start_time.elapsed());
                    }

                    return Err(());
                }

                return Err(());
            }

            if let Some(custom_clang) = self.config.get_custom_clang() {
                if self.handle_command(&mut self.build_clang_command(custom_clang)) {
                    return Ok(start_time.elapsed());
                }

                return Err(());
            }

            return Err(());
        }

        Ok(start_time.elapsed())
    }

    pub fn build_clang_command(&self, clang_path: &Path) -> Command {
        let mut clang_command: Command = Command::new(clang_path);

        clang_command.arg("-v");
        clang_command.args(self.files.iter());
        clang_command.args(self.config.get_args().iter());

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

pub fn get_x86_64_linux_clang() -> (&'static [u8], PathBuf, PathBuf) {
    (
        LINUX_X86_64_CLANG,
        PathBuf::from("clang.tar.xz"),
        PathBuf::from("clang-17"),
    )
}
