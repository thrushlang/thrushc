use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::typesystem::types::Type;

#[inline]
pub fn validate_binary(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match op {
        TokenType::Arith
        | TokenType::Star
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Plus => self::validate_binary_arithmetic(op, a, b, span),
        TokenType::Xor => self::validate_xor(op, a, b, span),
        TokenType::Bor => self::validate_bor(op, a, b, span),
        TokenType::BAnd => self::validate_band(op, a, b, span),
        TokenType::BangEq | TokenType::EqEq => self::validate_binary_equality(op, a, b, span),
        TokenType::LessEq | TokenType::Less | TokenType::GreaterEq | TokenType::Greater => {
            self::validate_binary_comparasion(op, a, b, span)
        }
        TokenType::LShift | TokenType::RShift => self::validate_binary_shift(op, a, b, span),
        TokenType::And | TokenType::Or => self::validate_binary_gate(op, a, b, span),

        _ => Err(CompilationIssue::Error(
            String::from("Unknown Type Operation"),
            format!("'{}{}' isn't valid operation.", op, a),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_band(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    if let (
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_bor(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    if let (
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_xor(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    if let (
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_binary_gate(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_binary_shift(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    if let (
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' is not allowed.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_binary_comparasion(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    if let (
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
        Type::S8
        | Type::S16
        | Type::S32
        | Type::S64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::FPPC128, Type::FPPC128) = (a, b) {
        return Ok(());
    } else if let (Type::FX8680, Type::FX8680) = (a, b) {
        return Ok(());
    } else if let (Type::F32 | Type::F64 | Type::F128, Type::F32 | Type::F64 | Type::F128) = (a, b)
    {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_binary_equality(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
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
                | Type::U64
                | Type::U128,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::U128,
        ) | (
            Type::F32 | Type::F64 | Type::F128,
            Type::F32 | Type::F64 | Type::F128
        ) | (Type::Bool, Type::Bool)
            | (Type::Char, Type::Char)
            | (Type::FX8680, Type::FX8680)
            | (Type::FPPC128, Type::FPPC128)
    ) {
        return Ok(());
    }

    if a.is_ptr_type() && b.is_ptr_type() {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{} {} {}' isn't valid operation.", a, op, b),
        None,
        span,
    ))
}

#[inline]
fn validate_binary_arithmetic(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
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

        (Type::FPPC128, Type::FPPC128) => Ok(()),
        (Type::FX8680, Type::FX8680) => Ok(()),
        (Type::F32 | Type::F64 | Type::F128, Type::F32 | Type::F64 | Type::F128) => Ok(()),

        _ => Err(CompilationIssue::Error(
            String::from("Incompatible Type Operation"),
            format!("'{} {} {}' isn't valid operation.", a, op, b),
            None,
            span,
        )),
    }
}
