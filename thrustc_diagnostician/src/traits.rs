use thrustc_errors::CompilationPosition;
use thrustc_logging::LoggingType;

use std::path::Path;

pub trait IssueDisassembler {
    fn get_title(&self) -> &str;
    fn get_logging_type(&self) -> LoggingType;
    fn get_path(&self) -> &Path;
    fn get_note(&self) -> Option<&str>;
}

pub trait ErrorDisassembler {
    fn get_title(&self) -> &str;
    fn get_position(&self) -> CompilationPosition;
    fn get_logging_type(&self) -> LoggingType;
    fn get_source_path(&self) -> &Path;
    fn get_compiler_source_path(&self) -> &Path;
    fn get_line(&self) -> u32;
}
