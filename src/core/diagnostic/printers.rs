use std::path::Path;

use colored::Colorize;

use crate::core::{
    console::logging::{self, LoggingType},
    diagnostic::{
        Diagnostic,
        errors::{BackendError, Error, FrontendError},
        traits::{FrontendErrorDisassembler, IssueDisassembler},
    },
    errors::position::CompilationPosition,
};

pub fn print(diagnostic: &Diagnostic, error: Error<'_>) {
    let title: &str = error.get_title();
    let path: &Path = error.get_path();
    let note: Option<&str> = error.get_note();
    let logging_type: LoggingType = error.get_logging_type();

    let code: &str = diagnostic.get_code();
    let signaler: &str = diagnostic.get_signaler();

    let line: usize = diagnostic.get_span().get_line();
    let start: usize = diagnostic.get_span().get_span_start();

    logging::write(
        logging::OutputIn::Stderr,
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

    logging::write(
        logging::OutputIn::Stderr,
        &format!("\n{} {}\n", logging_type.as_styled(), title.to_uppercase()),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!("\n{}\n{}", code, signaler),
    );

    if let Some(note) = note {
        logging::write(
            logging::OutputIn::Stderr,
            &format!("{} {}\n", "NOTE:".bright_blue().bold(), note),
        );
    }
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

    logging::write(
        logging::OutputIn::Stderr,
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

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\n{} {} {} {}{}{}{}{}\n",
            "FRONTEND BUG".bright_red().bold(),
            title.to_uppercase(),
            "-".bold(),
            compiler_source_path.display(),
            ":".bold(),
            position,
            ":".bold(),
            compiler_line.to_string().red().underline().bold()
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!("\n{}\n{}", code, signaler),
    );

    logging::write(
        logging::OutputIn::Stderr,
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

    logging::write(
        logging::OutputIn::Stderr,
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

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\n{} {} {} {}{}{}{}{}\n",
            "BACKEND BUG".bright_red().bold(),
            title.to_uppercase(),
            "-".bold(),
            position,
            ":".bold(),
            compiler_source_path.display(),
            ":".bold(),
            compiler_line.to_string().red().underline().bold()
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!("\n{}\n{}", code, signaler),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "Report it in '{}'.\n",
            "https://github.com/thrushlang/thrushc/issues"
                .white()
                .bold()
                .underline()
        ),
    );
}
