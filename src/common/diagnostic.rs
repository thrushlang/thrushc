use super::super::backend::compiler::misc::CompilerFile;

use super::{
    error::ThrushCompilerError,
    logging::{self, LoggingType},
};

use {
    colored::Colorize,
    std::{fs, path::PathBuf},
};

#[derive(Debug)]
pub struct Diagnostician {
    path: PathBuf,
    code: String,
}

#[derive(Debug)]
struct Diagnostic {
    code: String,
    signal: String,
}

#[derive(Debug)]
struct CodePosition {
    line: usize,
    start: usize,
    end: usize,
}

impl Diagnostician {
    pub fn new(thrushfile: &CompilerFile) -> Self {
        let code: String = fs::read_to_string(&thrushfile.path).unwrap_or_else(|_| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Unable to read `{}` file for build a diagnostic.",
                    thrushfile.path.display()
                ),
            );

            unreachable!()
        });

        Self {
            path: thrushfile.path.clone(),
            code,
        }
    }

    pub fn report_error(&mut self, error: &ThrushCompilerError, logging_type: LoggingType) {
        let ThrushCompilerError::Error(title, help, line, span) = error;
        self.print_error_report(title, help, *line, span.as_ref(), logging_type);
    }

    fn print_error_report(
        &mut self,
        title: &str,
        description: &str,
        line: usize,
        span: Option<&(usize, usize)>,
        logging_type: LoggingType,
    ) {
        self.print_header(line, title, logging_type);

        if let Some((start, end)) = span {
            Diagnostic::build(&self.code, line, *start, *end).print(description);
            return;
        }

        Diagnostic::build_without_span(&self.code, line).print(description);
    }

    fn print_header(&mut self, line: usize, title: &str, logging_type: LoggingType) {
        logging::write(
            logging::OutputIn::Stderr,
            format!(
                "{} at line {}\n",
                format_args!("{}", &self.path.to_string_lossy().bold().bright_red()),
                line.to_string().bold().bright_red()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{} {}\n\n", logging_type.to_styled(), title).as_bytes(),
        );
    }
}

impl Diagnostic {
    pub fn build(code: &str, line: usize, start: usize, end: usize) -> Self {
        if let Some(code_position) = Diagnostic::find_line_and_range(code, start, end) {
            if let Some(diagnostic) = Diagnostic::generate_diagnostic(code, code_position) {
                return diagnostic;
            }
        }

        Diagnostic::build_without_span(code, line)
    }

    pub fn find_line_and_range(code: &str, start: usize, end: usize) -> Option<CodePosition> {
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

    pub fn generate_diagnostic(code: &str, position: CodePosition) -> Option<Diagnostic> {
        let lines: Vec<&str> = code.lines().collect();

        if position.line > lines.len() {
            return None;
        }

        let line_position: usize = position.line - 1;

        let code_line: String = lines[line_position].to_string();
        let mut signal_line: String = " ".repeat(code_line.len() + 1);

        signal_line.replace_range(position.start + 1..position.end + 1, "—");

        Some(Diagnostic {
            code: code_line,
            signal: signal_line,
        })
    }

    pub fn build_without_span(code: &str, line: usize) -> Diagnostic {
        let lines: Vec<&str> = code.lines().collect();

        let code_line: String = "|  ".to_string() + lines[line - 1].trim();
        let signal_line: String = "|  ".to_string() + &"^".bright_red().repeat(code_line.len());

        Diagnostic {
            code: code_line,
            signal: signal_line,
        }
    }

    pub fn print(self, description: &str) {
        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{}\n{}\n", self.code, self.signal).as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{} {}\n\n", "> ".bold().bright_red(), description.bold()).as_bytes(),
        );
    }
}
