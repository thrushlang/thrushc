use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        typesystem::types::Type,
    },
};

pub fn validate_unary(
    operator: &TokenType,
    a: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match operator {
        TokenType::Minus | TokenType::PlusPlus | TokenType::MinusMinus => {
            self::validate_general_unary(operator, a, span)
        }

        TokenType::Bang => self::validate_unary_bang(a, span),

        _ => Ok(()),
    }
}

fn validate_general_unary(
    operator: &TokenType,
    a: &Type,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if a.is_integer_type() || a.is_float_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'{}' with '{}' isn't allowed.", operator, a),
        None,
        span,
    ))
}

fn validate_unary_bang(a: &Type, span: Span) -> Result<(), ThrushCompilerIssue> {
    if let Type::Bool = a {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Mismatched Types"),
        format!("'!{}' isn't allowed.", a),
        None,
        span,
    ))
}
