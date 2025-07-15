use std::fmt::Display;

use crate::core::compiler::options::CompilerFile;
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::{self, Diagnostic, printers};
use crate::core::errors::standard::ThrushCompilerIssue;

use {
    colored::Colorize,
    std::{fs, path::PathBuf},
};

#[derive(Debug, Clone, Copy)]
pub enum Notificator {
    CommonHelp,
    CompilerFronteEndBug,
}

#[derive(Debug, Clone)]
pub struct Diagnostician {
    path: PathBuf,
    code: String,
}

impl Diagnostician {
    pub fn new(file: &CompilerFile) -> Self {
        let code: String = fs::read_to_string(&file.path).unwrap_or_else(|_| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Unable to read '{}' file for a correct diagnostic.",
                    file.path.display()
                ),
            );

            unreachable!()
        });

        Self {
            path: file.path.clone(),
            code,
        }
    }
}

impl Diagnostician {
    pub fn build_diagnostic(&mut self, error: &ThrushCompilerIssue, logging_type: LoggingType) {
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

            ThrushCompilerIssue::FrontEndBug(title, info, span, position, line) => {
                let diagnostic: Diagnostic =
                    diagnostic::build(&self.code, *span, info, Notificator::CompilerFronteEndBug);

                printers::print_compiler_frontend_bug(
                    &diagnostic,
                    (title, *position, logging_type, &self.path, *line),
                );
            }
        };
    }
}

impl Diagnostician {
    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Notificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommonHelp => write!(f, "{}", "HELP: ".bright_green().bold()),
            Self::CompilerFronteEndBug => {
                write!(f, "{}", "COMPILER BUG INFO: ".bright_red().bold())
            }
        }
    }
}
