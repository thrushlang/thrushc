use std::path::{Path, PathBuf};

use target_triple::host;

use crate::{
    logging::{self, LoggingType},
    targets,
};

#[derive(Debug)]
pub struct BuildOptions {
    llvm_host_target: String,
    llvm_build_path: PathBuf,
}

impl BuildOptions {
    #[inline]
    pub fn new() -> BuildOptions {
        BuildOptions {
            llvm_host_target: host!().to_string(),

            llvm_build_path: if cfg!(target_os = "linux") {
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| {
                    logging::log(LoggingType::Panic, "Missing $HOME environment variable.\n");
                    std::process::exit(1);
                }))
            } else {
                PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| {
                    logging::log(
                        LoggingType::Panic,
                        "Missing $APPDATA environment variable.\n",
                    );
                    std::process::exit(1);
                }))
            }
            .join("thrushlang/backends/llvm/build"),
        }
    }
}

impl BuildOptions {
    #[inline]
    pub fn verify(&self) -> Result<(), String> {
        if targets::parse_llvm_triple(&self.llvm_host_target).is_none() {
            return Err(format!(
                "Host target '{}' is not a valid target for LLVM backend, try to specify it with '--target='.",
                self.llvm_host_target
            ));
        }

        Ok(())
    }
}

impl BuildOptions {
    #[inline]
    pub fn set_llvm_triple(&mut self, triple: String) -> Result<(), String> {
        if !targets::LLVM_HOSTS_TARGETS_AVAILIABLE.contains(&triple.as_str()) {
            return Err(format!(
                "Target '{}' is not a valid target for LLVM backend, try to guide with '--print-llvm-host-triples'.",
                triple
            ));
        }

        self.llvm_host_target = triple;

        Ok(())
    }
}

impl BuildOptions {
    #[inline]
    pub fn get_llvm_build_path(&self) -> &Path {
        &self.llvm_build_path
    }

    #[inline]
    pub fn get_llvm_host_triple(&self) -> String {
        format!("llvm-{}", &self.llvm_host_target)
    }

    #[inline]
    pub fn get_llvm_host_triple_versioned(&self) -> String {
        format!("llvm-{}-v", self.llvm_host_target)
    }
}
