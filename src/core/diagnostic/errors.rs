use std::path::Path;

use crate::core::{
    console::logging::LoggingType,
    diagnostic::traits::{FrontendErrorDisassembler, IssueDisassembler},
    errors::position::CompilationPosition,
};

pub type Error<'a> = (&'a str, &'a Path, Option<&'a str>, LoggingType);
pub type FrontendError<'a> = (&'a str, CompilationPosition, LoggingType, &'a Path, u32);

impl FrontendErrorDisassembler for FrontendError<'_> {
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
    fn get_path(&self) -> &Path {
        self.3
    }

    #[inline]
    fn get_line(&self) -> u32 {
        self.4
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
