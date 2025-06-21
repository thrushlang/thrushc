use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        types::lexer::ThrushType,
    },
};

pub fn validate_binary(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match operator {
        TokenType::Star | TokenType::Slash | TokenType::Minus | TokenType::Plus => {
            self::validate_binary_arithmetic(operator, a, b, span)
        }
        TokenType::BangEq | TokenType::EqEq => self::validate_binary_equality(operator, a, b, span),
        TokenType::LessEq | TokenType::Less | TokenType::GreaterEq | TokenType::Greater => {
            self::validate_binary_comparasion(operator, a, b, span)
        }
        TokenType::LShift | TokenType::RShift => self::validate_binary_shift(operator, a, b, span),
        TokenType::And | TokenType::Or => self::validate_binary_gate(operator, a, b, span),
        _ => Ok(()),
    }
}

fn validate_binary_gate(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (ThrushType::Bool, ThrushType::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("Logical ({} {} {}) isn't allowed.", a, operator, b),
        None,
        span,
    ))
}

fn validate_binary_shift(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (
        ThrushType::S8
        | ThrushType::S16
        | ThrushType::S32
        | ThrushType::S64
        | ThrushType::U8
        | ThrushType::U16
        | ThrushType::U32
        | ThrushType::U64,
        ThrushType::S8
        | ThrushType::S16
        | ThrushType::S32
        | ThrushType::S64
        | ThrushType::U8
        | ThrushType::U16
        | ThrushType::U32
        | ThrushType::U64,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("Arithmetic ({} {} {}) is not allowed.", a, operator, b),
        None,
        span,
    ))
}

fn validate_binary_comparasion(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (
        ThrushType::S8
        | ThrushType::S16
        | ThrushType::S32
        | ThrushType::S64
        | ThrushType::U8
        | ThrushType::U16
        | ThrushType::U32
        | ThrushType::U64,
        ThrushType::S8
        | ThrushType::S16
        | ThrushType::S32
        | ThrushType::S64
        | ThrushType::U8
        | ThrushType::U16
        | ThrushType::U32
        | ThrushType::U64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (ThrushType::F32 | ThrushType::F64, ThrushType::F32 | ThrushType::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("Logical ({} {} {}) isn't allowed.", a, operator, b),
        None,
        span,
    ))
}

fn validate_binary_equality(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if matches!(
        (a, b),
        (
            ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64,
            ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64,
        ) | (
            ThrushType::F32 | ThrushType::F64,
            ThrushType::F32 | ThrushType::F64
        ) | (ThrushType::Bool, ThrushType::Bool)
            | (ThrushType::Char, ThrushType::Char)
    ) {
        return Ok(());
    }

    if a.is_ptr_type() && b.is_ptr_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("Logical ({} {} {}) isn't allowed.", a, operator, b),
        None,
        span,
    ))
}

fn validate_binary_arithmetic(
    operator: &TokenType,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match (a, b) {
        (
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
        ) => Ok(()),

        (ThrushType::F32 | ThrushType::F64, ThrushType::F32 | ThrushType::F64) => Ok(()),

        _ => Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!("Arithmetic ({} {} {}) isn't allowed.", a, operator, b),
            None,
            span,
        )),
    }
}
