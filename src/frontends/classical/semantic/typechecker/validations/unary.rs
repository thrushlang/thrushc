use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        typesystem::types::Type,
    },
};

#[inline]
pub fn validate_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), ThrushCompilerIssue> {
    match op {
        TokenType::Minus | TokenType::PlusPlus | TokenType::MinusMinus => {
            self::validate_general_unary(op, a, span)
        }

        TokenType::Not => self::validate_not_unary(op, a, span),
        TokenType::Bang => self::validate_bang_unary(op, a, span),

        _ => Err(ThrushCompilerIssue::Error(
            String::from("Unknown Type Operation"),
            format!("'{}{}' isn't valid operation.", op, a),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_not_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), ThrushCompilerIssue> {
    if a.is_integer_type() || a.is_ptr_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}

#[inline]
fn validate_general_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), ThrushCompilerIssue> {
    if a.is_integer_type() || a.is_float_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}

#[inline]
fn validate_bang_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), ThrushCompilerIssue> {
    if let Type::Bool = a {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}
