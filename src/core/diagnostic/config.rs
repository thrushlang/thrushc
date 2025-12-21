use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiagnosticianConfig {
    export_path: PathBuf,
    export_errors: bool,
    export_warnings: bool,
}

impl DiagnosticianConfig {
    #[inline]
    pub fn new(export_path: PathBuf, export_errors: bool, export_warnings: bool) -> Self {
        Self {
            export_path,
            export_errors,
            export_warnings,
        }
    }
}

impl DiagnosticianConfig {
    #[inline]
    pub fn export_path(&self) -> &Path {
        &self.export_path
    }

    #[inline]
    pub fn export_errors(&self) -> bool {
        self.export_errors
    }

    #[inline]
    pub fn export_warnings(&self) -> bool {
        self.export_warnings
    }
}
