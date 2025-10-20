use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{lexer::span::Span, typesystem::types::Type},
};

pub fn float(lexeme: &str, span: Span) -> Result<(Type, f64), ThrushCompilerIssue> {
    if lexeme.bytes().filter(|&b| b == b'.').count() > 1 {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Only one decimal marker was expected.".into(),
            None,
            span,
        ));
    }

    lexeme
        .parse::<f32>()
        .map(|f| (Type::F32, f as f64))
        .or_else(|_| lexeme.parse::<f64>().map(|f| (Type::F64, f)))
        .map_err(|_| {
            ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Float out of bounds.".into(),
                None,
                span,
            )
        })
}

pub fn integer(lexeme: &str, span: Span) -> Result<(Type, u64), ThrushCompilerIssue> {
    const I8_MIN: isize = -128;
    const I8_MAX: isize = 127;
    const I16_MIN: isize = -32768;
    const I16_MAX: isize = 32767;
    const I32_MIN: isize = -2147483648;
    const I32_MAX: isize = 2147483647;

    const U8_MAX: usize = 255;
    const U16_MAX: usize = 65535;
    const U32_MAX: usize = 4294967295;

    fn match_signed(number: isize, span: Span) -> Result<(Type, u64), ThrushCompilerIssue> {
        match number {
            n if (I8_MIN..=I8_MAX).contains(&n) => Ok((Type::S8, n as u64)),
            n if (I16_MIN..=I16_MAX).contains(&n) => Ok((Type::S16, n as u64)),
            n if (I32_MIN..=I32_MAX).contains(&n) => Ok((Type::S32, n as u64)),
            n if (isize::MIN..=isize::MAX).contains(&n) => Ok((Type::S64, n as u64)),

            _ => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
        }
    }

    fn match_unsigned(number: usize, span: Span) -> Result<(Type, u64), ThrushCompilerIssue> {
        match number {
            n if (0..=U8_MAX).contains(&n) => Ok((Type::U8, n as u64)),
            n if (0..=U16_MAX).contains(&n) => Ok((Type::U16, n as u64)),
            n if (0..=U32_MAX).contains(&n) => Ok((Type::U32, n as u64)),
            n if (0..=usize::MAX).contains(&n) => Ok((Type::U64, n as u64)),

            _ => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
        }
    }

    let (radix, prefix, base) = if lexeme.starts_with("0x") {
        (16, "0x", "hexadecimal")
    } else if lexeme.starts_with("0b") {
        (2, "0b", "binary")
    } else {
        (10, "", "decimal")
    };

    if radix != 10 {
        let cleaned: String = lexeme
            .strip_prefix(prefix)
            .unwrap_or(lexeme)
            .replace('_', "");

        return isize::from_str_radix(&cleaned, radix)
            .map(|n| match_signed(n, span))
            .unwrap_or_else(|_| {
                usize::from_str_radix(&cleaned, radix)
                    .map(|n| match_unsigned(n, span))
                    .unwrap_or_else(|_| {
                        Err(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            format!("Integer invalid numeric '{}' format.", base),
                            None,
                            span,
                        ))
                    })
            });
    }

    lexeme
        .parse::<usize>()
        .map(|n| match_unsigned(n, span))
        .or_else(|_| lexeme.parse::<isize>().map(|n| match_signed(n, span)))
        .unwrap_or_else(|_| {
            Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Integer literal is too large to be represented in a integer type.".into(),
                None,
                span,
            ))
        })
}
