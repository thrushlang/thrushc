use std::path::PathBuf;

use crate::core::compiler::backends::llvm::utils;

#[derive(Debug)]
pub struct JITConfiguration {
    libc_path: PathBuf,
    libraries: Vec<PathBuf>,
    args: Vec<String>,
}

impl JITConfiguration {
    #[inline]
    pub fn new() -> Self {
        Self {
            libc_path: PathBuf::from(utils::get_default_dynamic_c_runtime()),
            libraries: Vec::with_capacity(10),
            args: Vec::with_capacity(10),
        }
    }
}

impl JITConfiguration {
    #[inline]
    pub fn get_libraries(&self) -> &[PathBuf] {
        &self.libraries
    }

    #[inline]
    pub fn get_libc_path(&self) -> &PathBuf {
        &self.libc_path
    }

    #[inline]
    pub fn get_args(&self) -> &[String] {
        &self.args
    }
}

impl JITConfiguration {
    #[inline]
    pub fn set_libc_path(&mut self, value: PathBuf) {
        self.libc_path = value;
    }

    #[inline]
    pub fn add_library(&mut self, value: PathBuf) {
        self.libraries.push(value);
    }

    #[inline]
    pub fn add_arg(&mut self, value: String) {
        self.args.push(value);
    }
}
