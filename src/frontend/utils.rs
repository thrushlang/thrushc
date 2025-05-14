use super::super::middle::types::Type;
use super::super::standard::error::ThrushCompilerIssue;
use super::lexer::Span;

pub fn parse_number(lexeme: &str, span: Span) -> Result<(Type, f64), ThrushCompilerIssue> {
    if lexeme.contains('.') {
        return parse_float(lexeme, span);
    }

    parse_integer(lexeme, span)
}

pub fn parse_float(lexeme: &str, span: Span) -> Result<(Type, f64), ThrushCompilerIssue> {
    let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

    if dot_count > 1 {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Float values should only contain one dot."),
            String::default(),
            span,
        ));
    }

    if let Ok(float) = lexeme.parse::<f32>() {
        return Ok((Type::F32, float.into()));
    }

    if let Ok(float) = lexeme.parse::<f64>() {
        return Ok((Type::F64, float));
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        String::from("Out of bounds."),
        String::default(),
        span,
    ))
}

pub fn parse_integer(lexeme: &str, span: Span) -> Result<(Type, f64), ThrushCompilerIssue> {
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

    if lexeme.starts_with("0x") {
        let cleaned_lexeme: String = lexeme
            .strip_prefix("0x")
            .unwrap_or(&lexeme.replace("0x", ""))
            .replace("_", "");

        return match isize::from_str_radix(&cleaned_lexeme, 16) {
            Ok(number) => {
                if (I8_MIN..=I8_MAX).contains(&number) {
                    return Ok((Type::S8, number as f64));
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    return Ok((Type::S16, number as f64));
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    return Ok((Type::S32, number as f64));
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    return Ok((Type::S64, number as f64));
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds signed hexadecimal format."),
                        String::default(),
                        span,
                    ));
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
                Ok(number) => {
                    if (U8_MIN..=U8_MAX).contains(&number) {
                        return Ok((Type::U8, number as f64));
                    } else if (U16_MIN..=U16_MAX).contains(&number) {
                        return Ok((Type::U16, number as f64));
                    } else if (U32_MIN..=U32_MAX).contains(&number) {
                        return Ok((Type::U32, number as f64));
                    } else if (usize::MIN..=usize::MAX).contains(&number) {
                        return Ok((Type::U64, number as f64));
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds unsigned hexadecimal format."),
                            String::default(),
                            span,
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Invalid numeric hexadecimal format."),
                    String::default(),
                    span,
                )),
            },
        };
    }

    if lexeme.starts_with("0b") {
        let cleaned_lexeme: String = lexeme
            .strip_prefix("0b")
            .unwrap_or(&lexeme.replace("0b", ""))
            .replace("_", "");

        return match isize::from_str_radix(&cleaned_lexeme, 2) {
            Ok(number) => {
                if (I8_MIN..=I8_MAX).contains(&number) {
                    return Ok((Type::U8, number as f64));
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    return Ok((Type::U16, number as f64));
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    return Ok((Type::U32, number as f64));
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    return Ok((Type::U64, number as f64));
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds signed binary format."),
                        String::default(),
                        span,
                    ));
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
                Ok(number) => {
                    if (U8_MIN..=U8_MAX).contains(&number) {
                        return Ok((Type::U8, number as f64));
                    } else if (U16_MIN..=U16_MAX).contains(&number) {
                        return Ok((Type::U16, number as f64));
                    } else if (U32_MIN..=U32_MAX).contains(&number) {
                        return Ok((Type::U32, number as f64));
                    } else if (usize::MIN..=usize::MAX).contains(&number) {
                        return Ok((Type::U64, number as f64));
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds unsigned binary format."),
                            String::default(),
                            span,
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Invalid binary format."),
                    String::default(),
                    span,
                )),
            },
        };
    }

    match lexeme.parse::<usize>() {
        Ok(number) => {
            if (U8_MIN..=U8_MAX).contains(&number) {
                Ok((Type::U8, number as f64))
            } else if (U16_MIN..=U16_MAX).contains(&number) {
                return Ok((Type::U16, number as f64));
            } else if (U32_MIN..=U32_MAX).contains(&number) {
                return Ok((Type::U32, number as f64));
            } else if (usize::MIN..=usize::MAX).contains(&number) {
                return Ok((Type::U64, number as f64));
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Out of bounds."),
                    String::default(),
                    span,
                ));
            }
        }

        Err(_) => match lexeme.parse::<isize>() {
            Ok(number) => {
                if (I8_MIN..=I8_MAX).contains(&number) {
                    Ok((Type::U8, number as f64))
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    Ok((Type::U16, number as f64))
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    Ok((Type::U32, number as f64))
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    Ok((Type::U64, number as f64))
                } else {
                    Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds."),
                        String::default(),
                        span,
                    ))
                }
            }

            Err(_) => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Out of bounds."),
                String::default(),
                span,
            )),
        },
    }
}
