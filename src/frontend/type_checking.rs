use crate::middle::instruction::Instruction;

use super::{
    super::{common::error::ThrushCompilerError, middle::types::*},
    lexer::Span,
};

#[inline(always)]
fn check_binary_arithmetic(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
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

        _ => Err(ThrushCompilerError::Error(
            String::from("Type checking"),
            format!("Arithmetic operation ({} {} {}) is not allowed.", a, op, b),
            span,
        )),
    }
}

#[inline(always)]
fn check_binary_equality(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
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

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        span,
    ))
}

#[inline(always)]
fn check_binary_comparasion(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
    if let (
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        span,
    ))
}

#[inline(always)]
fn check_binary_gate(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
    if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        span,
    ))
}

#[inline(always)]
fn check_binary_shift(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
    if let (
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Arithmetic operation ({} {} {}) is not allowed.", a, op, b),
        span,
    ))
}

#[inline(always)]
pub fn check_binary_types(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), ThrushCompilerError> {
    match op {
        TokenKind::Star | TokenKind::Slash | TokenKind::Minus | TokenKind::Plus => {
            check_binary_arithmetic(op, a, b, span)
        }
        TokenKind::BangEq | TokenKind::EqEq => check_binary_equality(op, a, b, span),
        TokenKind::LessEq | TokenKind::Less | TokenKind::GreaterEq | TokenKind::Greater => {
            check_binary_comparasion(op, a, b, span)
        }
        TokenKind::LShift | TokenKind::RShift => check_binary_shift(op, a, b, span),
        TokenKind::And | TokenKind::Or => check_binary_gate(op, a, b, span),
        _ => Ok(()),
    }
}

/*

UNARY INSTRUCTION

--------------------
OPERATOR OPERATOR
--------------------
*/

#[inline(always)]
fn check_unary(op: &TokenKind, a: &Type, span: Span) -> Result<(), ThrushCompilerError> {
    if a.is_integer_type() || a.is_float_type() {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Arithmetic operation '{}' with '{}' is not allowed.", op, a),
        span,
    ))
}

#[inline(always)]
fn check_unary_instr_bang(a: &Type, span: Span) -> Result<(), ThrushCompilerError> {
    if let Type::Bool = a {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation (!{}) is not allowed.", a),
        span,
    ))
}

#[inline(always)]
pub fn check_unary_types(op: &TokenKind, a: &Type, span: Span) -> Result<(), ThrushCompilerError> {
    match op {
        TokenKind::Minus | TokenKind::PlusPlus | TokenKind::MinusMinus => check_unary(op, a, span),
        TokenKind::Bang => check_unary_instr_bang(a, span),
        _ => Ok(()),
    }
}

pub fn check_type(
    target_type: &Type,
    from_type: &Type,
    expression: Option<&Instruction>,
    operator: Option<&TokenKind>,
    error: ThrushCompilerError,
) -> Result<(), ThrushCompilerError> {
    if let Some(Instruction::BinaryOp {
        operator,
        kind: expression_type,
        ..
    }) = expression
    {
        return check_type(target_type, expression_type, None, Some(operator), error);
    }

    if let Some(Instruction::UnaryOp {
        operator,
        kind: expression_type,
        ..
    }) = expression
    {
        return check_type(target_type, expression_type, None, Some(operator), error);
    }

    if let Some(Instruction::Group {
        expression,
        kind: expression_type,
        ..
    }) = expression
    {
        return check_type(target_type, expression_type, Some(expression), None, error);
    }

    match (target_type, from_type, operator) {
        (Type::Char, Type::Char, None) => Ok(()),
        (Type::Str, Type::Str, None) => Ok(()),
        (Type::Struct(_, target_fields), Type::Struct(_, from_fields), None) => {
            if target_fields.len() != from_fields.len() {
                return Err(error);
            }

            target_fields.iter().zip(from_fields.iter()).try_for_each(
                |(target_field, from_field)| {
                    check_type(target_field, from_field, None, None, error.clone())
                },
            )?;

            Ok(())
        }

        (Type::Struct(_, _), Type::Void, None) => Ok(()),

        (Type::Ptr(None), Type::Ptr(None), None) => Ok(()),
        (Type::Ptr(Some(target_type)), Type::Ptr(Some(from_type)), None) => {
            check_type(target_type, from_type, expression, operator, error)?;
            Ok(())
        }

        (Type::Ptr(Some(typed)), any, None) => {
            check_type(typed, any, expression, operator, error)?;
            Ok(())
        }

        (
            Type::Bool,
            Type::Bool,
            Some(
                TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Bang,
            )
            | None,
        ) => Ok(()),
        (
            Type::S8,
            Type::S8 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::S16,
            Type::S16 | Type::S8 | Type::U16 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::S32,
            Type::S32 | Type::S16 | Type::S8 | Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::S64,
            Type::S64
            | Type::S32
            | Type::S16
            | Type::S8
            | Type::U64
            | Type::U32
            | Type::U16
            | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::U8,
            Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::U16,
            Type::U16 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::U32,
            Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::U64,
            Type::U64 | Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::F32,
            Type::F32,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::F64,
            Type::F64 | Type::F32,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),

        _ => Err(error),
    }
}
