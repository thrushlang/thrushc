pub mod diagnostician;
pub mod errors;
pub mod position;
pub mod printers;

use colored::Colorize;
use std::str::Lines;

use crate::{
    core::diagnostic::{diagnostician::Notificator, position::CodePosition},
    frontend::lexer::span::Span,
};

#[derive(Debug)]
pub struct Diagnostic<'a> {
    code: &'a str,
    signaler: String,
    span: Span,
}

impl<'a> Diagnostic<'a> {
    pub fn new(code: &'a str, signaler: String, span: Span) -> Self {
        Self {
            code,
            signaler,
            span,
        }
    }
}

impl Diagnostic<'_> {
    pub fn get_code(&self) -> &str {
        self.code
    }

    pub fn get_signaler(&self) -> &str {
        &self.signaler
    }

    pub fn get_span(&self) -> Span {
        self.span
    }
}

pub fn build<'a>(
    code: &'a str,
    span: Span,
    info: &'a str,
    notificator: Notificator,
) -> Diagnostic<'a> {
    if let Some(code_position) = position::find_line_and_range(code, span) {
        if let Some(diagnostic) = self::generate(code, code_position, info, notificator) {
            return diagnostic;
        }
    }

    self::generate_basic(code, span, info)
}

pub fn generate_basic<'a>(code: &'a str, span: Span, info: &'a str) -> Diagnostic<'a> {
    let lines: Vec<&str> = code.lines().collect();

    let line: usize = span.line;

    let code: &str = lines[line - 1].trim_start();
    let mut signaler: String = String::with_capacity(100);

    for i in 0..=code.len() {
        if i == code.len() {
            signaler.push_str(&format!("\n\n{}{}\n", "HELP: ".bright_green().bold(), info));
            break;
        }

        signaler.push('^');
    }

    Diagnostic::new(code, signaler, span)
}

pub fn generate<'a>(
    code: &'a str,
    position: CodePosition,
    info: &str,
    notificator: Notificator,
) -> Option<Diagnostic<'a>> {
    let mut lines: Lines = code.lines();

    let pivot_line: usize = position.get_line();
    let code_line: &str = lines.nth(pivot_line.saturating_sub(1))?;

    let line: usize = position.get_line();
    let start: usize = position.get_start();
    let end: usize = position.get_end();

    let code_before_trim: usize = code_line.len();
    let code: &str = code_line.trim_start();

    let trim_diferrence: usize = code_before_trim - code.len();

    let mut signaler: String = String::with_capacity(100);

    let end_position: usize = end.saturating_sub(trim_diferrence).saturating_sub(1);

    for pos in 0..=end_position {
        if pos == end_position {
            signaler.push('^');
            signaler.push(' ');
            signaler.push_str(&format!("{}{}", notificator, info));
            signaler.push_str("\n\n");

            break;
        }

        signaler.push(' ');
    }

    Some(Diagnostic::new(
        code,
        signaler,
        Span::new(line, (start, end)),
    ))
}
