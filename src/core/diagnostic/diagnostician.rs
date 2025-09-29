use std::fmt::Display;

use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::{self, Diagnostic, printers};
use crate::core::errors::standard::ThrushCompilerIssue;

use {colored::Colorize, std::path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum Notificator {
    CommonHelp,
    CompilerFrontendBug,
    CompilerBackendBug,
}

#[derive(Debug, Clone)]
pub struct Diagnostician {
    path: PathBuf,
    code: String,
}

impl Diagnostician {
    #[inline]
    pub fn new(file: &CompilationUnit) -> Self {
        Self {
            path: file.get_path().to_path_buf(),
            code: file.get_unit_clone(),
        }
    }
}

impl Diagnostician {
    pub fn dispatch_diagnostic(&mut self, error: &ThrushCompilerIssue, logging_type: LoggingType) {
        match error {
            ThrushCompilerIssue::Error(title, help, note, span) => {
                let diagnostic: Diagnostic =
                    diagnostic::build(&self.code, *span, help, Notificator::CommonHelp);

                printers::print(
                    &diagnostic,
                    (title, &self.path, note.as_deref(), logging_type),
                );
            }

            ThrushCompilerIssue::Warning(title, help, span) => {
                let diagnostic: Diagnostic =
                    diagnostic::build(&self.code, *span, help, Notificator::CommonHelp);

                printers::print(&diagnostic, (title, &self.path, None, logging_type));
            }

            ThrushCompilerIssue::FrontEndBug(title, info, span, position, path, line) => {
                let diagnostic: Diagnostic =
                    diagnostic::build(&self.code, *span, info, Notificator::CompilerFrontendBug);

                printers::print_compiler_frontend_bug(
                    &diagnostic,
                    (title, *position, logging_type, &self.path, path, *line),
                );
            }

            ThrushCompilerIssue::BackenEndBug(title, info, span, position, path, line) => {
                let diagnostic: Diagnostic =
                    diagnostic::build(&self.code, *span, info, Notificator::CompilerBackendBug);

                printers::print_compiler_backend_bug(
                    &diagnostic,
                    (title, *position, logging_type, &self.path, path, *line),
                );
            }
        };
    }
}

impl Diagnostician {
    #[inline]
    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Notificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommonHelp => write!(f, "{}", "HELP: ".bright_green().bold()),
            Self::CompilerFrontendBug | Self::CompilerBackendBug => {
                write!(f, "{}", "INFO: ".bright_red().bold())
            }
        }
    }
}
