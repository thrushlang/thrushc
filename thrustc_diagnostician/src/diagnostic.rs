use colored::Colorize;

use thrustc_logging::LoggingType;
use thrustc_span::Span;

use crate::{Notificator, position::CodePosition};

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
}

impl<'a> Diagnostic<'a> {
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
pub(crate) fn build<'a>(
    code: &'a str,
    span: Span,
    message: &'a str,
    notificator: Notificator,
    logging_type: LoggingType,
) -> Diagnostic<'a> {
    match crate::position::find_line_and_range(code, span) {
        Some(code_position) => {
            self::generate(code, code_position, message, notificator, logging_type)
                .unwrap_or_else(|| self::generate_basic(code, span, message, notificator))
        }
        None => self::generate_basic(code, span, message, notificator),
    }
}

pub(crate) fn generate_basic<'a>(
    code: &'a str,
    span: Span,
    message: &'a str,
    notificator: Notificator,
) -> Diagnostic<'a> {
    let lines: Vec<&str> = code.lines().collect();
    let line_idx: usize = span.line.saturating_sub(1);

    let code_line: &str = lines.get(line_idx).map(|s| s.trim_start()).unwrap_or("");

    let mut signaler: String = String::with_capacity(256);

    if line_idx > 0 {
        if let Some(prev_line) = lines.get(line_idx - 1) {
            signaler.push_str(&format!(
                "{:>4} │ {}\n",
                span.line - 1,
                prev_line.bright_black()
            ));
        }
    }

    signaler.push_str(&format!(
        "{:>4} │ {}\n",
        span.line,
        code_line.bright_white().bold()
    ));

    signaler.push_str(&format!(
        "{:>4} │ {}\n",
        "",
        "~".repeat(code_line.len()).bright_red().bold()
    ));

    signaler.push_str(&format!("{}{}\n", notificator, message.bright_yellow()));

    if let Some(next_line) = lines.get(line_idx + 1) {
        signaler.push_str(&format!(
            "{:>4} │ {}\n",
            span.line + 1,
            next_line.bright_black()
        ));
    }

    signaler.push('\n');

    Diagnostic::new(code_line, signaler, span)
}

pub(crate) fn generate<'a>(
    code: &'a str,
    position: CodePosition,
    message: &'a str,
    notificator: Notificator,
    logging_type: LoggingType,
) -> Option<Diagnostic<'a>> {
    let lines: Vec<&str> = code.lines().collect();
    let line_idx: usize = position.get_line().saturating_sub(1);

    let code_line: &str = lines.get(line_idx)?.trim_start();
    let code_before_trim: usize = lines.get(line_idx)?.len();
    let trim_difference: usize = code_before_trim.saturating_sub(code_line.len());

    let line: usize = position.get_line();
    let start: usize = position.get_start().saturating_sub(trim_difference);
    let end: usize = position.get_end().saturating_sub(trim_difference);

    if start > end || end > code_line.len() {
        return None;
    }

    let mut signaler: String = String::with_capacity(256);

    if line_idx > 0 {
        if let Some(prev_line) = lines.get(line_idx - 1) {
            signaler.push_str(&format!("{:>4} │ {}\n", line - 1, prev_line.bright_black()));
        }
    }

    signaler.push_str(&format!(
        "{:>4} │ {}\n",
        line,
        code_line.bright_white().bold()
    ));

    signaler.push_str(&format!("{:>4} │ ", ""));

    for i in 0..code_line.len() {
        if i >= start && i < end {
            signaler.push_str(&logging_type.text_with_color("^").to_string());
        } else {
            signaler.push(' ');
        }
    }

    signaler.push_str(&format!("{}{}\n", notificator, message.bright_yellow()));

    if let Some(next_line) = lines.get(line_idx + 1) {
        signaler.push_str(&format!("{:>4} │ {}\n", line + 1, next_line.bright_black()));
    }

    signaler.push('\n');

    Some(Diagnostic::new(
        code_line,
        signaler,
        Span::new(line, (start, end)),
    ))
}
