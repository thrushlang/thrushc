use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_typesystem::Type;

pub fn floating_point(lexeme: &str, span: Span) -> Result<(Type, f64), CompilationIssue> {
    if lexeme.bytes().filter(|&b| b == b'.').count() > 1 {
        Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Floating-point number only expects a one decimal marker.".into(),
            None,
            span,
        ))
    } else {
        lexeme
            .parse::<f32>()
            .map(|f| (Type::F32(span), f as f64))
            .or_else(|_| lexeme.parse::<f64>().map(|f| (Type::F64(span), f)))
            .map_err(|_| {
                CompilationIssue::Error(
                    CompilationIssueCode::E0001,
                    "Literal is too large to be represented in a standard floating-point type."
                        .into(),
                    None,
                    span,
                )
            })
    }
}

pub fn integer(lexeme: &str, span: Span) -> Result<(Type, u64), CompilationIssue> {
    const I8_MIN: isize = -128;
    const I8_MAX: isize = 127;
    const I16_MIN: isize = -32768;
    const I16_MAX: isize = 32767;
    const I32_MIN: isize = -2147483648;
    const I32_MAX: isize = 2147483647;

    const U8_MAX: usize = 255;
    const U16_MAX: usize = 65535;
    const U32_MAX: usize = 4294967295;

    fn match_signed(number: isize, span: Span) -> Result<(Type, u64), CompilationIssue> {
        match number {
            n if (I8_MIN..=I8_MAX).contains(&n) => Ok((Type::S8(span), n as u64)),
            n if (I16_MIN..=I16_MAX).contains(&n) => Ok((Type::S16(span), n as u64)),
            n if (I32_MIN..=I32_MAX).contains(&n) => Ok((Type::S32(span), n as u64)),
            n if (isize::MIN..=isize::MAX).contains(&n) => Ok((Type::S64(span), n as u64)),

            _ => Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
        }
    }

    fn match_unsigned(number: usize, span: Span) -> Result<(Type, u64), CompilationIssue> {
        match number {
            n if (0..=U8_MAX).contains(&n) => Ok((Type::U8(span), n as u64)),
            n if (0..=U16_MAX).contains(&n) => Ok((Type::U16(span), n as u64)),
            n if (0..=U32_MAX).contains(&n) => Ok((Type::U32(span), n as u64)),
            n if (0..=usize::MAX).contains(&n) => Ok((Type::U64(span), n as u64)),

            _ => Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Literal is too large to be represented in a integer type.".into(),
                None,
                span,
            )),
        }
    }

    let hexadecimal: bool = lexeme.strip_prefix("0x").is_some();
    let octal: bool = lexeme.strip_prefix("0o").is_some();
    let binary: bool = lexeme.strip_prefix("0b").is_some();

    let (radix, prefix, base) = if hexadecimal {
        (16, "0x", "hexadecimal")
    } else if octal {
        (8, "0o", "octal")
    } else if binary {
        (2, "0b", "binary")
    } else {
        (10, "", "decimal")
    };

    if radix != 10 {
        let cleaned: String = lexeme
            .strip_prefix(prefix)
            .unwrap_or(lexeme)
            .replace('_', "");

        isize::from_str_radix(&cleaned, radix)
            .map(|n| match_signed(n, span))
            .unwrap_or_else(|_| {
                usize::from_str_radix(&cleaned, radix)
                    .map(|n| match_unsigned(n, span))
                    .unwrap_or_else(|_| {
                        Err(CompilationIssue::Error(
                            CompilationIssueCode::E0001,
                            format!("Integer invalid numeric '{}' format.", base),
                            None,
                            span,
                        ))
                    })
            })
    } else {
        lexeme
            .parse::<usize>()
            .map(|n| match_unsigned(n, span))
            .or_else(|_| lexeme.parse::<isize>().map(|n| match_signed(n, span)))
            .unwrap_or_else(|_| {
                Err(CompilationIssue::Error(
                    CompilationIssueCode::E0001,
                    "Literal is too large to be represented in a integer type.".into(),
                    None,
                    span,
                ))
            })
    }
}
