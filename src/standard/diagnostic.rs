use std::path::Path;
use std::str::Lines;

use super::{
    super::frontend::lexer::Span,
    error::ThrushCompilerIssue,
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

    pub fn build_diagnostic(&mut self, error: &ThrushCompilerIssue, logging_type: LoggingType) {
        match error {
            ThrushCompilerIssue::Error(title, help, note, span) => {
                self.diagnose(title, help, Some(note), *span, logging_type);
            }
            ThrushCompilerIssue::Warning(title, help, span) => {
                self.diagnose(title, help, None, *span, logging_type);
            }

            _ => todo!(),
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
        Diagnostic::build(&self.code, span, help).print(&self.path, title, note, logging_type);
    }
}

impl<'a> Diagnostic<'a> {
    pub fn build(code: &'a str, span: Span, help: &'a str) -> Self {
        if let Some(code_position) = Diagnostic::find_line_and_range(code, span) {
            if let Some(diagnostic) = Diagnostic::generate_diagnostic(code, code_position, help) {
                return diagnostic;
            }
        }

        Diagnostic::build_without_span(code, span, help)
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
        code: &'a str,
        position: CodePosition,
        help: &str,
    ) -> Option<Diagnostic<'a>> {
        let mut lines: Lines = code.lines();

        let line: &str = lines.nth(position.line.saturating_sub(1))?;

        let code_before_trim: usize = line.len();
        let code: &str = line.trim_start();

        let trim_diferrence: usize = code_before_trim - code.len();

        let mut signaler: String = String::with_capacity(100);

        for pos in 0..=position.end - trim_diferrence {
            if pos == position.end - trim_diferrence {
                signaler.push('^');
                signaler.push(' ');
                signaler.push_str(&format!(
                    "{}{}",
                    "HELP: ".bright_green().bold(),
                    help.bold()
                ));
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

    pub fn build_without_span(code: &'a str, span: Span, help: &'a str) -> Diagnostic<'a> {
        let lines: Vec<&str> = code.lines().collect();

        let line: usize = span.line;

        let code: &str = lines[line - 1].trim_start();
        let mut signaler: String = String::with_capacity(200);

        for i in 0..=code.len() {
            if i == code.len() {
                signaler.push_str(&format!(
                    "\n\n{}{}\n",
                    "HELP: ".bright_green().bold(),
                    help.bold()
                ));

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
                "{} {} at {}:{}\n",
                "-->".bold().blink(),
                format_args!(
                    "{}",
                    logging_type.text_with_color(path.to_string_lossy().as_ref())
                ),
                logging_type.text_with_color(&self.span.get_line().to_string()),
                logging_type.text_with_color(&self.span.get_span_start().to_string()),
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "\n{} {}\n\n",
                logging_type.to_styled(),
                title.bold().to_uppercase()
            ),
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
}
