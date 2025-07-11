use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        typesystem::types::Type,
    },
};

pub fn validate_binary(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match op {
        TokenType::Star | TokenType::Slash | TokenType::Minus | TokenType::Plus => {
            self::validate_binary_arithmetic(op, a, b, span)
        }
        TokenType::BangEq | TokenType::EqEq => self::validate_binary_equality(op, a, b, span),
        TokenType::LessEq | TokenType::Less | TokenType::GreaterEq | TokenType::Greater => {
            self::validate_binary_comparasion(op, a, b, span)
        }
        TokenType::LShift | TokenType::RShift => self::validate_binary_shift(op, a, b, span),
        TokenType::And | TokenType::Or => self::validate_binary_gate(op, a, b, span),

        _ => Ok(()),
    }
}

fn validate_binary_gate(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'{} {} {}' isn't allowed.", a, op, b),
        None,
        span,
    ))
}

fn validate_binary_shift(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'{} {} {}' is not allowed.", a, op, b),
        None,
        span,
    ))
}

fn validate_binary_comparasion(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'{} {} {}' isn't allowed.", a, op, b),
        None,
        span,
    ))
}

fn validate_binary_equality(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if matches!(
        (a, b),
        (
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64,
        ) | (Type::F32 | Type::F64, Type::F32 | Type::F64)
            | (Type::Bool, Type::Bool)
            | (Type::Char, Type::Char)
    ) {
        return Ok(());
    }

    if a.is_ptr_type() && b.is_ptr_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'{} {} {}' isn't allowed.", a, op, b),
        None,
        span,
    ))
}

fn validate_binary_arithmetic(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match (a, b) {
        (
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64,
        ) => Ok(()),

        (Type::F32 | Type::F64, Type::F32 | Type::F64) => Ok(()),

        _ => Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!("'{} {} {}' isn't allowed.", a, op, b),
            None,
            span,
        )),
    }
}
