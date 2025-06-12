use std::str::Lines;
use std::{fmt::Display, path::Path};

use crate::core::compiler::options::CompilerFile;
use crate::core::console::logging::{self, LoggingType};
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;
use crate::frontend::lexer::span::Span;

use {
    colored::Colorize,
    std::{fs, path::PathBuf},
};

#[derive(Debug, Clone, Copy)]
pub enum NotificatorType {
    CommonHelp,
    CompilerBug,
}

#[derive(Debug, Clone)]
pub struct Diagnostician {
    path: PathBuf,
    code: String,
}

#[derive(Debug)]
struct Diagnostic<'a> {
    code: &'a str,
    signaler: String,
    span: Span,
}

#[derive(Debug)]
struct CodePosition {
    line: usize,
    start: usize,
    end: usize,
}

impl Diagnostician {
    pub fn new(file: &CompilerFile) -> Self {
        let code: String = fs::read_to_string(&file.path).unwrap_or_else(|_| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Unable to read `{}` file for build a diagnostic.",
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

    pub fn build_diagnostic(&mut self, error: &ThrushCompilerIssue, logging_type: LoggingType) {
        match error {
            ThrushCompilerIssue::Error(title, help, note, span) => {
                self.diagnose(title, help, note.as_deref(), *span, logging_type);
            }
            ThrushCompilerIssue::Warning(title, help, span) => {
                self.diagnose(title, help, None, *span, logging_type);
            }
            ThrushCompilerIssue::Bug(title, info, span, position, line) => {
                Diagnostic::build(&self.code, *span, info, NotificatorType::CompilerBug)
                    .print_compiler_bug(title, *position, LoggingType::Bug, &self.path, *line);
            }
        }
    }

    fn diagnose(
        &mut self,
        title: &str,
        help: &str,
        note: Option<&str>,
        span: Span,
        logging_type: LoggingType,
    ) {
        Diagnostic::build(&self.code, span, help, NotificatorType::CommonHelp).print(
            &self.path,
            title,
            note,
            logging_type,
        );
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl<'a> Diagnostic<'a> {
    pub fn build(
        code: &'a str,
        span: Span,
        info: &'a str,
        notificator_type: NotificatorType,
    ) -> Self {
        if let Some(code_position) = Diagnostic::find_line_and_range(code, span) {
            if let Some(diagnostic) =
                Diagnostic::generate_diagnostic(code, code_position, info, notificator_type)
            {
                return diagnostic;
            }
        }

        Diagnostic::build_without_span(code, span, info)
    }

    pub fn find_line_and_range(code: &str, span: Span) -> Option<CodePosition> {
        let start: usize = span.get_span_start();
        let end: usize = span.get_span_end();

        let mut line_start: usize = 0;
        let mut line_num: usize = 1;

        for (i, c) in code.char_indices() {
            if i >= start {
                break;
            }
            if c == '\n' {
                line_start = i + 1;
                line_num += 1;
            }
        }

        if start >= code.len() || end > code.len() || start > end {
            return None;
        }

        Some(CodePosition {
            line: line_num,
            start: start.saturating_sub(line_start),
            end: end.saturating_sub(line_start),
        })
    }

    pub fn generate_diagnostic(
        code: &'a str,
        position: CodePosition,
        info: &str,
        notificator_type: NotificatorType,
    ) -> Option<Diagnostic<'a>> {
        let mut lines: Lines = code.lines();

        let line: &str = lines.nth(position.line.saturating_sub(1))?;

        let code_before_trim: usize = line.len();
        let code: &str = line.trim_start();

        let trim_diferrence: usize = code_before_trim - code.len();

        let mut signaler: String = String::with_capacity(100);

        let end_position = position
            .end
            .saturating_sub(trim_diferrence)
            .saturating_sub(1);

        for pos in 0..=end_position {
            if pos == end_position {
                signaler.push('^');
                signaler.push(' ');
                signaler.push_str(&format!("{}{}", notificator_type, info));
                signaler.push_str("\n\n");
                break;
            }

            signaler.push(' ');
        }

        Some(Diagnostic {
            code,
            signaler,
            span: Span::new(position.line, (position.start, position.end)),
        })
    }

    pub fn build_without_span(code: &'a str, span: Span, info: &'a str) -> Diagnostic<'a> {
        let lines: Vec<&str> = code.lines().collect();

        let line: usize = span.line;

        let code: &str = lines[line - 1].trim_start();
        let mut signaler: String = String::with_capacity(200);

        for i in 0..=code.len() {
            if i == code.len() {
                signaler.push_str(&format!("\n\n{}{}\n", "HELP: ".bright_green().bold(), info));
                break;
            }

            signaler.push('^');
        }

        Diagnostic {
            code,
            signaler,
            span,
        }
    }

    pub fn print(self, path: &Path, title: &str, note: Option<&str>, logging_type: LoggingType) {
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
                logging_type.text_with_color(&self.span.get_line().to_string()),
                logging_type.text_with_color(&self.span.get_span_start().to_string()),
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!("\n{} {}\n", logging_type.to_styled(), title.to_uppercase()),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!("\n{}\n{}", self.code, self.signaler),
        );

        if let Some(note) = note {
            logging::write(
                logging::OutputIn::Stderr,
                &format!("{} {}\n", "NOTE:".bright_blue().bold(), note),
            );
        }
    }

    pub fn print_compiler_bug(
        self,
        title: &str,
        position: CompilationPosition,
        logging_type: LoggingType,
        path: &Path,
        line: u32,
    ) {
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
                logging_type.text_with_color(&self.span.get_line().to_string()),
                logging_type.text_with_color(&self.span.get_span_start().to_string()),
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "\n{} {} {} {}{}{}\n",
                logging_type.to_styled(),
                title.to_uppercase(),
                "-".bold(),
                position,
                ":".bold(),
                line.to_string().red().underline().bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!("\n{}\n{}", self.code, self.signaler),
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
}

impl Display for NotificatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommonHelp => write!(f, "{}", "HELP: ".bright_green().bold()),
            Self::CompilerBug => write!(f, "{}", "INFO: ".bright_red().bold()),
        }
    }
}
