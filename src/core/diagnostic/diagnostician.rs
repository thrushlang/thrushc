use std::fmt::Display;
use std::path::Path;

use crate::core::compiler;
use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::span::Span;
use crate::core::diagnostic::{self, Diagnostic, printers};
use crate::core::errors::standard::CompilationIssue;
use crate::front_end::preprocessor::errors::PreprocessorIssue;

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
    pub fn dispatch_diagnostic(&mut self, error: &CompilationIssue, logging_type: LoggingType) {
        match error {
            CompilationIssue::Error(title, help, note, span) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    help,
                    Notificator::CommonHelp,
                    logging_type,
                );

                printers::print(
                    &diagnostic,
                    (title, &self.path, note.as_deref(), logging_type),
                );
            }

            CompilationIssue::Warning(title, help, span) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    help,
                    Notificator::CommonHelp,
                    logging_type,
                );

                printers::print(&diagnostic, (title, &self.path, None, logging_type));
            }

            CompilationIssue::FrontEndBug(title, info, span, position, path, line) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    info,
                    Notificator::CompilerFrontendBug,
                    logging_type,
                );

                printers::print_compiler_frontend_bug(
                    &diagnostic,
                    (title, *position, logging_type, &self.path, path, *line),
                );
            }

            CompilationIssue::BackenEndBug(title, info, span, position, path, line) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    info,
                    Notificator::CompilerBackendBug,
                    logging_type,
                );

                printers::print_compiler_backend_bug(
                    &diagnostic,
                    (title, *position, logging_type, &self.path, path, *line),
                );
            }
        };
    }
}

impl Diagnostician {
    pub fn dispatch_preprocessor_diagnostic(
        &mut self,
        error: &PreprocessorIssue,
        logging_type: LoggingType,
    ) {
        let path: &Path = error.get_path();
        let title: &str = error.get_title();
        let description: &str = error.get_description();
        let span: Span = error.get_span();

        let source: String = compiler::reader::get_file_source_code(path);

        let diagnostic: Diagnostic = diagnostic::build(
            &source,
            span,
            description,
            Notificator::CommonHelp,
            logging_type,
        );

        printers::print(&diagnostic, (title, path, None, logging_type));
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
            Self::CommonHelp => write!(f, "{}", " HELP: ".bright_green().bold()),
            Self::CompilerFrontendBug | Self::CompilerBackendBug => {
                write!(f, "{}", " INFO: ".bright_red().bold())
            }
        }
    }
}
