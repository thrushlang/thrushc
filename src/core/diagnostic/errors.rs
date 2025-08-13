use std::path::Path;

use crate::core::{console::logging::LoggingType, errors::position::CompilationPosition};

pub type Issue<'a> = (&'a str, &'a Path, Option<&'a str>, LoggingType);
pub type FrontendError<'a> = (&'a str, CompilationPosition, LoggingType, &'a Path, u32);

pub trait FrontendErrorDisassembler {
    fn get_title(&self) -> &str;
    fn get_position(&self) -> CompilationPosition;
    fn get_logging_type(&self) -> LoggingType;
    fn get_path(&self) -> &Path;
    fn get_line(&self) -> u32;
}

impl FrontendErrorDisassembler for FrontendError<'_> {
    fn get_title(&self) -> &str {
        self.0
    }

    fn get_position(&self) -> CompilationPosition {
        self.1
    }

    fn get_logging_type(&self) -> LoggingType {
        self.2
    }

    fn get_path(&self) -> &Path {
        self.3
    }

    fn get_line(&self) -> u32 {
        self.4
    }
}

pub trait IssueDisassembler {
    fn get_title(&self) -> &str;
    fn get_logging_type(&self) -> LoggingType;
    fn get_path(&self) -> &Path;
    fn get_note(&self) -> Option<&str>;
}

impl IssueDisassembler for Issue<'_> {
    fn get_title(&self) -> &str {
        self.0
    }

    fn get_path(&self) -> &Path {
        self.1
    }

    fn get_note(&self) -> Option<&str> {
        self.2
    }

    fn get_logging_type(&self) -> LoggingType {
        self.3
    }
}
