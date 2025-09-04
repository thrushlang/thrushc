use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::lexer::{Lexer, span::Span},
};

const I8_MIN: isize = -128;
const I8_MAX: isize = 127;
const I16_MIN: isize = -32768;
const I16_MAX: isize = 32767;
const I32_MIN: isize = -2147483648;
const I32_MAX: isize = 2147483647;

const U8_MIN: usize = 0;
const U8_MAX: usize = 255;
const U16_MIN: usize = 0;
const U16_MAX: usize = 65535;
const U32_MIN: usize = 0;
const U32_MAX: usize = 4294967295;

#[inline]
pub fn check_integer_format(lexer: &Lexer, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
    let span: Span = Span::new(lexer.line, lexer.span);

    if lexeme.starts_with("0x") {
        return self::check_integer_hex_format(lexeme, span);
    }

    if lexeme.starts_with("0b") {
        return self::check_integer_binary_format(lexeme, span);
    }

    match lexeme.parse::<usize>() {
        Ok(num) => {
            if (U8_MIN..=U8_MAX).contains(&num)
                || (U16_MIN..=U16_MAX).contains(&num)
                || (U32_MIN..=U32_MAX).contains(&num)
                || (usize::MIN..=usize::MAX).contains(&num)
            {
                Ok(())
            } else {
                Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer out of bounds.".into(),
                    None,
                    span,
                ))
            }
        }

        Err(_) => match lexeme.parse::<isize>() {
            Ok(num) => {
                if (I8_MIN..=I8_MAX).contains(&num)
                    || (I16_MIN..=I16_MAX).contains(&num)
                    || (I32_MIN..=I32_MAX).contains(&num)
                    || (isize::MIN..=isize::MAX).contains(&num)
                {
                    Ok(())
                } else {
                    Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Integer out of bounds.".into(),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer out of bounds.".into(),
                None,
                span,
            )),
        },
    }
}

fn check_integer_binary_format(lexeme: &str, span: Span) -> Result<(), ThrushCompilerIssue> {
    let cleaned_lexeme: String = lexeme
        .strip_prefix("0b")
        .unwrap_or(&lexeme.replace("0b", ""))
        .replace("_", "");

    match isize::from_str_radix(&cleaned_lexeme, 2) {
        Ok(num) => {
            if (I8_MIN..=I8_MAX).contains(&num)
                || (I16_MIN..=I16_MAX).contains(&num)
                || (I32_MIN..=I32_MAX).contains(&num)
                || (isize::MIN..=isize::MAX).contains(&num)
            {
                Ok(())
            } else {
                Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer out of bounds signed binary format.".into(),
                    None,
                    span,
                ))
            }
        }

        Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
            Ok(num) => {
                if (U8_MIN..=U8_MAX).contains(&num)
                    || (U16_MIN..=U16_MAX).contains(&num)
                    || (U32_MIN..=U32_MAX).contains(&num)
                    || (usize::MIN..=usize::MAX).contains(&num)
                {
                    Ok(())
                } else {
                    Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Integer out of bounds unsigned binary format.".into(),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer invalid binary format.".into(),
                None,
                span,
            )),
        },
    }
}

fn check_integer_hex_format(lexeme: &str, span: Span) -> Result<(), ThrushCompilerIssue> {
    let cleaned_lexeme: String = lexeme
        .strip_prefix("0x")
        .unwrap_or(&lexeme.replace("0x", ""))
        .replace("_", "");

    match isize::from_str_radix(&cleaned_lexeme, 16) {
        Ok(num) => {
            if (I8_MIN..=I8_MAX).contains(&num)
                || (I16_MIN..=I16_MAX).contains(&num)
                || (I32_MIN..=I32_MAX).contains(&num)
                || (isize::MIN..=isize::MAX).contains(&num)
            {
                Ok(())
            } else {
                Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer out of bounds signed hexadecimal format.".into(),
                    None,
                    span,
                ))
            }
        }

        Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
            Ok(num) => {
                if (U8_MIN..=U8_MAX).contains(&num)
                    || (U16_MIN..=U16_MAX).contains(&num)
                    || (U32_MIN..=U32_MAX).contains(&num)
                    || (usize::MIN..=usize::MAX).contains(&num)
                {
                    Ok(())
                } else {
                    Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Integer out of bounds unsigned hexadecimal format.".into(),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer invalid numeric hexadecimal format.".into(),
                None,
                span,
            )),
        },
    }
}
