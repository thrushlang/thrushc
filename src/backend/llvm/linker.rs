use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use lld::{LLDResult, LldFlavor};

use crate::standard::logging;

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

        let lld_flags: Vec<&str> = self.setup_flags();

        let lld_result: LLDResult = lld::link_all(LldFlavor::Elf, lld_flags);

        if !lld_result.get_state() {
            logging::log(logging::LoggingType::Error, lld_result.get_messages());
        }

        lld_time.elapsed()
    }

    fn setup_flags(&self) -> Vec<&str> {
        let mut flags: Vec<&str> = Vec::with_capacity(self.flags.len() + self.files.len());

        flags.extend(
            self.files
                .iter()
                .map(|file_path| {
                    file_path.to_str().unwrap_or_else(|| {
                        logging::log(
                            logging::LoggingType::Error,
                            &format!(
                                "Failed to convert path to valid str utf-8 during link time: '{}'.",
                                file_path.display()
                            ),
                        );

                        ""
                    })
                })
                .collect::<Vec<&str>>(),
        );

        flags.extend(self.flags.split(";"));

        flags
    }
}
