pub mod diagnostician;
pub mod errors;
pub mod position;
pub mod printers;
pub mod traits;

use colored::Colorize;
use std::str::Lines;

use crate::{
    core::diagnostic::{diagnostician::Notificator, position::CodePosition},
    front_end::lexer::span::Span,
};

#[derive(Debug)]
pub struct Diagnostic<'a> {
    code: &'a str,
    signaler: String,
    span: Span,
}

impl<'a> Diagnostic<'a> {
    #[inline]
    pub fn new(code: &'a str, signaler: String, span: Span) -> Self {
        Self {
            code,
            signaler,
            span,
        }
    }

    #[inline]
    pub fn get_code(&self) -> &str {
        self.code
    }

    #[inline]
    pub fn get_signaler(&self) -> &str {
        &self.signaler
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }
}

#[inline]
pub fn build<'a>(
    code: &'a str,
    span: Span,
    message: &'a str,
    notificator: Notificator,
) -> Diagnostic<'a> {
    match position::find_line_and_range(code, span) {
        Some(code_position) => self::generate(code, code_position, message, notificator)
            .unwrap_or_else(|| self::generate_basic(code, span, message)),
        None => self::generate_basic(code, span, message),
    }
}

pub fn generate_basic<'a>(code: &'a str, span: Span, message: &'a str) -> Diagnostic<'a> {
    let lines: Vec<&str> = code.lines().collect();
    let line_idx: usize = span.line.saturating_sub(1);

    let code_line: &str = lines.get(line_idx).map(|s| s.trim_start()).unwrap_or("");
    let mut signaler: String = String::with_capacity(128);

    signaler.push_str(&format!(
        "{:>4} │ {}\n",
        span.line,
        code_line.bright_white()
    ));

    signaler.push_str(&format!(
        "{:>4} │ {}\n",
        "",
        "^".repeat(code_line.len()).bright_red()
    ));

    signaler.push_str(&format!(
        "{} {}\n\n",
        "HELP:".bright_green().bold(),
        message.bright_yellow()
    ));

    Diagnostic::new(code_line, signaler, span)
}

pub fn generate<'a>(
    code: &'a str,
    position: CodePosition,
    message: &'a str,
    notificator: Notificator,
) -> Option<Diagnostic<'a>> {
    let lines: Lines = code.lines();
    let line_idx: usize = position.get_line().saturating_sub(1);

    let code_line: &str = lines.clone().nth(line_idx)?.trim_start();
    let code_before_trim: usize = lines.clone().nth(line_idx)?.len();
    let trim_difference: usize = code_before_trim.saturating_sub(code_line.len());

    let line: usize = position.get_line();
    let start: usize = position.get_start().saturating_sub(trim_difference);
    let end: usize = position.get_end().saturating_sub(trim_difference);

    if start > end || end > code_line.len() {
        return None;
    }

    let mut signaler: String = String::with_capacity(128);

    signaler.push_str(&format!("{:>4} │ {}\n", line, code_line.bright_white()));
    signaler.push_str(&format!("{:>4} │ ", ""));

    for i in 0..code_line.len() {
        if i >= start && i < end {
            signaler.push_str(&"^".bright_red());
        } else {
            signaler.push(' ');
        }
    }

    signaler.push_str(&format!(
        "{}{}\n",
        notificator.to_string().bright_cyan().bold(),
        message.bright_yellow()
    ));

    signaler.push('\n');

    Some(Diagnostic::new(
        code_line,
        signaler,
        Span::new(line, (start, end)),
    ))
}
