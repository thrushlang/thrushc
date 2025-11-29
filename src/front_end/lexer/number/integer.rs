use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::Lexer;

const I8_MIN: isize = -128;
const I8_MAX: isize = 127;
const I16_MIN: isize = -32768;
const I16_MAX: isize = 32767;
const I32_MIN: isize = -2147483648;
const I32_MAX: isize = 2147483647;

const U8_MAX: usize = 255;
const U16_MAX: usize = 65535;
const U32_MAX: usize = 4294967295;

#[inline]
pub fn check_integer_format(lexer: &Lexer, lexeme: &str) -> Result<(), CompilationIssue> {
    let span: Span = Span::new(lexer.line, lexer.span);

    if let Some(rest) = lexeme.strip_prefix("0x") {
        return self::check_integer_radix_format(rest, 16, span);
    }

    if let Some(rest) = lexeme.strip_prefix("0b") {
        return self::check_integer_radix_format(rest, 2, span);
    }

    let cleaned: String = lexeme.replace('_', "");

    match cleaned.parse::<usize>() {
        Ok(num) if num <= U8_MAX || num <= U16_MAX || num <= U32_MAX || num < usize::MAX => Ok(()),
        Ok(_) => Err(CompilationIssue::Error(
            "Syntax error".into(),
            "Integer literal is too large to be represented in a integer type.".into(),
            None,
            span,
        )),
        Err(_) => match cleaned.parse::<isize>() {
            Ok(num)
                if (I8_MIN..=I8_MAX).contains(&num)
                    || (I16_MIN..=I16_MAX).contains(&num)
                    || (I32_MIN..=I32_MAX).contains(&num)
                    || (isize::MIN..=isize::MAX).contains(&num) =>
            {
                Ok(())
            }
            Ok(_) => Err(CompilationIssue::Error(
                "Syntax error".into(),
                "Integer literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
            Err(_) => Err(CompilationIssue::Error(
                "Syntax error".into(),
                "Integer literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
        },
    }
}

fn check_integer_radix_format(
    lexeme: &str,
    radix: u32,
    span: Span,
) -> Result<(), CompilationIssue> {
    let cleaned: String = lexeme.replace('_', "");

    match isize::from_str_radix(&cleaned, radix) {
        Ok(num)
            if (I8_MIN..=I8_MAX).contains(&num)
                || (I16_MIN..=I16_MAX).contains(&num)
                || (I32_MIN..=I32_MAX).contains(&num)
                || (isize::MIN..=isize::MAX).contains(&num) =>
        {
            Ok(())
        }
        Ok(_) => Err(CompilationIssue::Error(
            "Syntax error".into(),
            format!(
                "Integer out of bounds signed {} format.",
                if radix == 16 { "hexadecimal" } else { "binary" }
            ),
            None,
            span,
        )),
        Err(_) => match usize::from_str_radix(&cleaned, radix) {
            Ok(num) if num <= U8_MAX || num <= U16_MAX || num <= U32_MAX || num < usize::MAX => {
                Ok(())
            }
            Ok(_) => Err(CompilationIssue::Error(
                "Syntax error".into(),
                format!(
                    "Integer out of bounds unsigned {} format.",
                    if radix == 16 { "hexadecimal" } else { "binary" }
                ),
                None,
                span,
            )),
            Err(_) => Err(CompilationIssue::Error(
                "Syntax error".into(),
                format!(
                    "Integer invalid numeric {} format.",
                    if radix == 16 { "hexadecimal" } else { "binary" }
                ),
                None,
                span,
            )),
        },
    }
}
