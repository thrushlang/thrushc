use crate::core::compiler;
use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::config::DiagnosticianConfig;
use crate::core::diagnostic::span::Span;
use crate::core::diagnostic::{self, Diagnostic, printers};
use crate::core::errors::standard::CompilationIssue;
use crate::front_end::preprocessor::errors::PreprocessorIssue;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum Notificator {
    CommonHelp,
    CompilerFrontendBug,
    CompilerBackendBug,
}

#[derive(Debug, Clone)]
pub struct Diagnostician {
    path: PathBuf,
    base_name: String,
    code: String,
    config: DiagnosticianConfig,
}

impl Diagnostician {
    #[inline]
    pub fn new(file: &CompilationUnit, options: &CompilerOptions) -> Self {
        Self {
            path: file.get_path().to_path_buf(),
            base_name: file.get_base_name(),
            code: file.get_unit_clone(),
            config: DiagnosticianConfig::new(
                options.get_export_diagnostics_path().to_path_buf(),
                options.get_export_compiler_error_diagnostics(),
                options.get_export_compiler_warning_diagnostics(),
            ),
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

                let generated: String = printers::print_to_string(
                    &diagnostic,
                    (
                        title,
                        &self.path,
                        note.as_ref().map(|x| x.as_str()),
                        logging_type,
                    ),
                );

                if self.get_config().export_errors() {
                    let path: PathBuf = self.get_config().export_path().join("errors");

                    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "Unable to create errors diagnostics path for export purposes!",
                        );
                    });

                    let full_path: PathBuf = path.join(format!("{}.txt", self.get_base_name()));

                    if let Ok(mut file_diag) =
                        OpenOptions::new().create(true).append(true).open(full_path)
                    {
                        let _ = file_diag.write(generated.as_bytes());
                    }
                }

                logging::write(logging::OutputIn::Stderr, &generated);
            }

            CompilationIssue::Warning(title, help, span) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    help,
                    Notificator::CommonHelp,
                    logging_type,
                );

                let generated: String =
                    printers::print_to_string(&diagnostic, (title, &self.path, None, logging_type));

                if self.get_config().export_warnings() {
                    let path: PathBuf = self.get_config().export_path().join("warnings");

                    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "Unable to create warnings diagnostics path for export purposes!",
                        );
                    });

                    let full_path: PathBuf = path.join(format!("{}.txt", self.get_base_name()));

                    if let Ok(mut file_diag) =
                        OpenOptions::new().create(true).append(true).open(full_path)
                    {
                        let _ = file_diag.write(generated.as_bytes());
                    }
                }

                logging::write(logging::OutputIn::Stderr, &generated);
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

        let diag: String =
            printers::print_to_string(&diagnostic, (title, &self.path, None, logging_type));

        logging::write(logging::OutputIn::Stderr, &diag);
    }
}

impl Diagnostician {
    #[inline]
    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }

    #[inline]
    pub fn get_config(&self) -> &DiagnosticianConfig {
        &self.config
    }

    #[inline]
    pub fn get_base_name(&self) -> &str {
        &self.base_name
    }
}
