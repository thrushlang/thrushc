pub mod config;
pub mod diagnostic;
pub mod errors;
mod impls;
mod position;
mod printers;
mod traits;

use thrustc_errors::CompilationIssue;
use thrustc_logging::LoggingType;
use thrustc_logging::OutputIn;
use thrustc_options::CompilationUnit;
use thrustc_options::CompilerOptions;

use crate::config::DiagnosticianConfig;
use crate::diagnostic::Diagnostic;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
enum Notificator {
    CommonHelp,
    CompilerFrontendBug,
    CompilerBackendBug,
}

#[derive(Debug, Clone, Default)]
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
                        &title.to_title(),
                        &self.path,
                        note.as_ref().map(|x| x.as_str()),
                        logging_type,
                    ),
                );

                if self.get_config().export_errors() {
                    let path: PathBuf = self.get_config().export_path().join("errors");

                    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
                        thrustc_logging::print_warn(
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

                thrustc_logging::write(OutputIn::Stderr, &generated);
            }

            CompilationIssue::Warning(title, help, span) => {
                let diagnostic: Diagnostic = diagnostic::build(
                    &self.code,
                    *span,
                    help,
                    Notificator::CommonHelp,
                    logging_type,
                );

                let generated: String = printers::print_to_string(
                    &diagnostic,
                    (&title.to_title(), &self.path, None, logging_type),
                );

                if self.get_config().export_warnings() {
                    let path: PathBuf = self.get_config().export_path().join("warnings");

                    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
                        thrustc_logging::print_warn(
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

                thrustc_logging::write(OutputIn::Stderr, &generated);
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
