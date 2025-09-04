use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{lexer::span::Span, typesystem::types::Type},
};

pub fn float(lexeme: &str, span: Span) -> Result<(Type, f64), ThrushCompilerIssue> {
    let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

    if dot_count > 1 {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Only one decimal marker was expected.".into(),
            None,
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
        "Syntax error".into(),
        "Float out of bounds.".into(),
        None,
        span,
    ))
}

pub fn integer(lexeme: &str, span: Span) -> Result<(Type, u64), ThrushCompilerIssue> {
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
                    return Ok((Type::S8, number as u64));
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    return Ok((Type::S16, number as u64));
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    return Ok((Type::S32, number as u64));
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    return Ok((Type::S64, number as u64));
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Integer out of bounds signed hexadecimal format.".into(),
                        None,
                        span,
                    ));
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
                Ok(number) => {
                    if (U8_MIN..=U8_MAX).contains(&number) {
                        return Ok((Type::U8, number as u64));
                    } else if (U16_MIN..=U16_MAX).contains(&number) {
                        return Ok((Type::U16, number as u64));
                    } else if (U32_MIN..=U32_MAX).contains(&number) {
                        return Ok((Type::U32, number as u64));
                    } else if (usize::MIN..=usize::MAX).contains(&number) {
                        return Ok((Type::U64, number as u64));
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            "Integer out of bounds unsigned hexadecimal format.".into(),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer invalid numeric hexadecimal format.".into(),
                    None,
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
                    return Ok((Type::U8, number as u64));
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    return Ok((Type::U16, number as u64));
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    return Ok((Type::U32, number as u64));
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    return Ok((Type::U64, number as u64));
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Integer out of bounds signed binary format.".into(),
                        None,
                        span,
                    ));
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
                Ok(number) => {
                    if (U8_MIN..=U8_MAX).contains(&number) {
                        return Ok((Type::U8, number as u64));
                    } else if (U16_MIN..=U16_MAX).contains(&number) {
                        return Ok((Type::U16, number as u64));
                    } else if (U32_MIN..=U32_MAX).contains(&number) {
                        return Ok((Type::U32, number as u64));
                    } else if (usize::MIN..=usize::MAX).contains(&number) {
                        return Ok((Type::U64, number as u64));
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            "Integer out of bounds unsigned binary format.".into(),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer invalid binary format.".into(),
                    None,
                    span,
                )),
            },
        };
    }

    match lexeme.parse::<usize>() {
        Ok(number) => {
            if (U8_MIN..=U8_MAX).contains(&number) {
                Ok((Type::U8, number as u64))
            } else if (U16_MIN..=U16_MAX).contains(&number) {
                return Ok((Type::U16, number as u64));
            } else if (U32_MIN..=U32_MAX).contains(&number) {
                return Ok((Type::U32, number as u64));
            } else if (usize::MIN..=usize::MAX).contains(&number) {
                return Ok((Type::U64, number as u64));
            } else {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Integer out of bounds.".into(),
                    None,
                    span,
                ));
            }
        }

        Err(_) => match lexeme.parse::<isize>() {
            Ok(number) => {
                if (I8_MIN..=I8_MAX).contains(&number) {
                    Ok((Type::U8, number as u64))
                } else if (I16_MIN..=I16_MAX).contains(&number) {
                    Ok((Type::U16, number as u64))
                } else if (I32_MIN..=I32_MAX).contains(&number) {
                    Ok((Type::U32, number as u64))
                } else if (isize::MIN..=isize::MAX).contains(&number) {
                    Ok((Type::U64, number as u64))
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
