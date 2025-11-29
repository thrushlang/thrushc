use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::typesystem::types::Type;

#[inline]
pub fn validate_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    match op {
        TokenType::Minus | TokenType::PlusPlus | TokenType::MinusMinus => {
            self::validate_general_unary(op, a, span)
        }

        TokenType::Not => self::validate_not_unary(op, a, span),
        TokenType::Bang => self::validate_bang_unary(op, a, span),

        _ => Err(CompilationIssue::Error(
            String::from("Unknown Type Operation"),
            format!("'{}{}' isn't valid operation.", op, a),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_not_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    if a.is_integer_type() || a.is_ptr_type() {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}

#[inline]
fn validate_general_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    if a.is_integer_type() || a.is_float_type() {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}

#[inline]
fn validate_bang_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    if let Type::Bool = a {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        String::from("Incompatible Type Operation"),
        format!("'{}{}' isn't valid operation.", op, a),
        None,
        span,
    ))
}
