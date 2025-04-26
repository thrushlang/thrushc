use std::path::Path;
use std::str::Lines;

use super::{
    super::frontend::lexer::Span,
    error::ThrushCompilerError,
    logging::{self, LoggingType},
    misc::CompilerFile,
};

use {
    colored::Colorize,
    std::{fs, path::PathBuf},
};

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

    pub fn build_diagnostic(&mut self, error: &ThrushCompilerError, logging_type: LoggingType) {
        let ThrushCompilerError::Error(title, help, span) = error;
        self.diagnose(title, help, *span, logging_type);
    }

    fn diagnose(&mut self, title: &str, description: &str, span: Span, logging_type: LoggingType) {
        Diagnostic::build(title, &self.code, span).print(
            &self.path,
            title,
            logging_type,
            description,
        );
    }
}

impl<'a> Diagnostic<'a> {
    pub fn build(title: &'a str, code: &'a str, span: Span) -> Self {
        if let Some(code_position) = Diagnostic::find_line_and_range(code, span) {
            if let Some(diagnostic) = Diagnostic::generate_diagnostic(title, code, code_position) {
                return diagnostic;
            }
        }

        Diagnostic::build_without_span(code, span)
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
            start: start - line_start,
            end: end - line_start,
        })
    }

    pub fn generate_diagnostic(
        title: &'a str,
        code: &'a str,
        position: CodePosition,
    ) -> Option<Diagnostic<'a>> {
        let mut lines: Lines = code.lines();

        let line: &str = lines.nth(position.line.saturating_sub(1))?;

        let code_before_trim: usize = line.len();
        let code: &str = line.trim_start();

        let trim_diferrence: usize = code_before_trim - code.len();

        let mut signaler: String = String::with_capacity(100);

        let fixer_arrow_position: usize = if !title.to_lowercase().contains("syntax error") {
            1
        } else {
            0
        };

        for pos in 0..=position.end - trim_diferrence - fixer_arrow_position {
            if pos == position.end - trim_diferrence - fixer_arrow_position {
                signaler.push('^');
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

    pub fn build_without_span(code: &str, span: Span) -> Diagnostic {
        let lines: Vec<&str> = code.lines().collect();

        let line: usize = span.line;

        let code: &str = lines[line - 1].trim_start();
        let signaler: String = "^".bright_red().repeat(code.len());

        Diagnostic {
            code,
            signaler,
            span,
        }
    }

    pub fn print(self, path: &Path, title: &str, logging_type: LoggingType, description: &str) {
        logging::write(
            logging::OutputIn::Stderr,
            format!(
                "{} at {}:{}\n",
                format_args!("{}", path.to_string_lossy().bold().bright_red()),
                self.span.get_line().to_string().bold().bright_red(),
                self.span.get_span_start().to_string().bold().bright_red()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{} {}\n\n", logging_type.to_styled(), title).as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{}\n{}\n", self.code, self.signaler).as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{} {}\n\n", "> ".bold().bright_red(), description.bold()).as_bytes(),
        );
    }
}
