use crate::traits::ErrorDisassembler;
use crate::traits::IssueDisassembler;

use thrushc_errors::CompilationPosition;
use thrushc_logging::LoggingType;

use std::path::Path;

pub type Error<'a> = (&'a str, &'a Path, Option<&'a str>, LoggingType);

pub type FrontendError<'a> = (
    &'a str,
    CompilationPosition,
    LoggingType,
    &'a Path,
    &'a Path,
    u32,
);

pub type BackendError<'a> = (
    &'a str,
    CompilationPosition,
    LoggingType,
    &'a Path,
    &'a Path,
    u32,
);

impl ErrorDisassembler for FrontendError<'_> {
    #[inline]
    fn get_title(&self) -> &str {
        self.0
    }

    #[inline]
    fn get_position(&self) -> CompilationPosition {
        self.1
    }

    #[inline]
    fn get_logging_type(&self) -> LoggingType {
        self.2
    }

    #[inline]
    fn get_source_path(&self) -> &Path {
        self.3
    }

    #[inline]
    fn get_compiler_source_path(&self) -> &Path {
        self.4
    }

    #[inline]
    fn get_line(&self) -> u32 {
        self.5
    }
}

impl IssueDisassembler for Error<'_> {
    #[inline]
    fn get_title(&self) -> &str {
        self.0
    }

    #[inline]
    fn get_path(&self) -> &Path {
        self.1
    }

    #[inline]
    fn get_note(&self) -> Option<&str> {
        self.2
    }

    #[inline]
    fn get_logging_type(&self) -> LoggingType {
        self.3
    }
}
