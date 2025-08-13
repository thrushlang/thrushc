use std::path::PathBuf;

#[derive(Debug)]
pub struct JITConfiguration {
    libc_path: Option<PathBuf>,
    libraries: Vec<PathBuf>,
}

impl JITConfiguration {
    pub fn new() -> Self {
        Self {
            libc_path: None,
            libraries: Vec::with_capacity(10),
        }
    }

    #[inline]
    pub fn set_libc_path(&mut self, value: PathBuf) {
        self.libc_path = Some(value);
    }

    #[inline]
    pub fn add_jit_library(&mut self, value: PathBuf) {
        self.libraries.push(value);
    }
}
