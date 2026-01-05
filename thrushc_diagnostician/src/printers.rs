use crate::diagnostic::Diagnostic;
use crate::errors::BackendError;
use crate::errors::Error;
use crate::errors::FrontendError;
use crate::traits::ErrorDisassembler;
use crate::traits::IssueDisassembler;

use thrushc_errors::CompilationPosition;
use thrushc_logging::OutputIn;
use thrushc_logging::{self, LoggingType};

use colored::Colorize;
use std::path::Path;

pub fn print_to_string(diagnostic: &Diagnostic, error: Error<'_>) -> String {
    let title: &str = error.get_title();
    let path: &Path = error.get_path();
    let note: Option<&str> = error.get_note();
    let logging_type: LoggingType = error.get_logging_type();

    let code: &str = diagnostic.get_code();
    let signaler: &str = diagnostic.get_signaler();

    let line: usize = diagnostic.get_span().get_line();
    let end: usize = diagnostic.get_span().get_span_end();

    let mut buffer = String::new();

    buffer.push_str(&format!(
        "{} {}:{}\n",
        format_args!(
            "{}",
            logging_type
                .text_with_color(path.to_string_lossy().as_ref())
                .underline()
        ),
        logging_type.text_with_color(&line.to_string()),
        logging_type.text_with_color(&end.to_string()),
    ));

    buffer.push_str(&format!("\n{}\n", title.to_uppercase()));
    buffer.push_str(&format!("\n{}\n{}", code, signaler));

    if let Some(note) = note {
        buffer.push_str(&format!("{} {}\n", "NOTE:".bright_blue().bold(), note));
    }

    buffer
}

pub fn print_compiler_frontend_bug(diagnostic: &Diagnostic, error: FrontendError<'_>) {
    let title: &str = error.get_title();
    let position: CompilationPosition = error.get_position();
    let compiler_line: u32 = error.get_line();
    let path: &Path = error.get_source_path();
    let compiler_source_path: &Path = error.get_compiler_source_path();
    let logging_type: LoggingType = error.get_logging_type();

    let code: &str = diagnostic.get_code();
    let signaler: &str = diagnostic.get_signaler();

    let line: usize = diagnostic.get_span().get_line();
    let start: usize = diagnostic.get_span().get_span_start();

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "{} {}:{}\n",
            format_args!(
                "{}",
                logging_type
                    .text_with_color(path.to_string_lossy().as_ref())
                    .underline()
            ),
            logging_type.text_with_color(&line.to_string()),
            logging_type.text_with_color(&start.to_string()),
        ),
    );

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "\n{} {} {} {} {}{}{}\n",
            "FRONTEND BUG".bright_red().bold(),
            title.to_uppercase(),
            "-".bold(),
            position,
            compiler_source_path.display(),
            ":".bold(),
            compiler_line.to_string().red().underline().bold()
        ),
    );

    thrushc_logging::write(OutputIn::Stderr, &format!("\n{}\n{}", code, signaler));

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "Report it in '{}'.\n",
            "https://github.com/thrushlang/thrushc/issues"
                .white()
                .bold()
                .underline()
        ),
    );
}

pub fn print_compiler_backend_bug(diagnostic: &Diagnostic, error: BackendError<'_>) {
    let title: &str = error.get_title();
    let position: CompilationPosition = error.get_position();
    let compiler_line: u32 = error.get_line();
    let path: &Path = error.get_source_path();
    let compiler_source_path: &Path = error.get_compiler_source_path();
    let logging_type: LoggingType = error.get_logging_type();

    let code: &str = diagnostic.get_code();
    let signaler: &str = diagnostic.get_signaler();

    let line: usize = diagnostic.get_span().get_line();
    let start: usize = diagnostic.get_span().get_span_start();

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "{} - {}:{}\n",
            format_args!(
                "{}",
                logging_type
                    .text_with_color(path.to_string_lossy().as_ref())
                    .underline()
            ),
            logging_type.text_with_color(&line.to_string()),
            logging_type.text_with_color(&start.to_string()),
        ),
    );

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "\n{} {} {} {} {}{}{}\n",
            "BACKEND BUG".bright_red().bold(),
            title.to_uppercase(),
            "-".bold(),
            position,
            compiler_source_path.display(),
            ":".bold(),
            compiler_line.to_string().red().underline().bold()
        ),
    );

    thrushc_logging::write(OutputIn::Stderr, &format!("\n{}\n{}", code, signaler));

    thrushc_logging::write(
        OutputIn::Stderr,
        &format!(
            "Report it in '{}'.\n",
            "https://github.com/thrushlang/thrushc/issues"
                .white()
                .bold()
                .underline()
        ),
    );
}
