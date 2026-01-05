use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;

use thrushc_token::tokentype::TokenType;
use thrushc_typesystem::{Type, traits::TypeIsExtensions};

#[inline]
pub fn validate_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    match op {
        TokenType::Minus | TokenType::PlusPlus | TokenType::MinusMinus => {
            self::validate_general_unary(op, a, span)
        }

        TokenType::Not => self::validate_not_unary(op, a, span),
        TokenType::Bang => self::validate_bang_unary(op, a, span),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0031,
            format!(
                "'{}{}' isn't a valid arithmetic or logical operation.",
                op, a
            ),
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
        CompilationIssueCode::E0030,
        format!("'{}{}' isn't a logical valid operation.", op, a),
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
        CompilationIssueCode::E0030,
        format!("'{}{}' isn't a valid arithmetic operation.", op, a),
        None,
        span,
    ))
}

#[inline]
fn validate_bang_unary(op: &TokenType, a: &Type, span: Span) -> Result<(), CompilationIssue> {
    if a.is_bool_type() || a.is_ptr_type() {
        return Ok(());
    }

    Err(CompilationIssue::Error(
        CompilationIssueCode::E0030,
        format!("'{}{}' isn't a valid logical operation.", op, a),
        None,
        span,
    ))
}
