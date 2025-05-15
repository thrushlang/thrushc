use crate::middle::types::frontend::{
    lexer::{tokenkind::TokenKind, types::ThrushType},
    parser::stmts::instruction::Instruction,
};

use super::{super::standard::error::ThrushCompilerIssue, lexer::Span};

pub fn check_binaryop(
    operator: &TokenKind,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match operator {
        TokenKind::Star | TokenKind::Slash | TokenKind::Minus | TokenKind::Plus => {
            check_binary_arithmetic(operator, a, b, span)
        }
        TokenKind::BangEq | TokenKind::EqEq => check_binary_equality(operator, a, b, span),
        TokenKind::LessEq | TokenKind::Less | TokenKind::GreaterEq | TokenKind::Greater => {
            check_binary_comparasion(operator, a, b, span)
        }
        TokenKind::LShift | TokenKind::RShift => check_binary_shift(operator, a, b, span),
        TokenKind::And | TokenKind::Or => check_binary_gate(operator, a, b, span),
        _ => Ok(()),
    }
}

fn check_binary_arithmetic(
    operator: &TokenKind,
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
        (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) => {
            check_binary_arithmetic(operator, a_subtype, b_subtype, span)
        }
        (any, ThrushType::Mut(b_subtype)) => {
            check_binary_arithmetic(operator, any, b_subtype, span)
        }
        (ThrushType::Mut(a_subtype), any) => {
            check_binary_arithmetic(operator, a_subtype, any, span)
        }

        _ => Err(ThrushCompilerIssue::Error(
            String::from("Type checking"),
            format!(
                "Arithmetic operation ({} {} {}) is not allowed.",
                a, operator, b
            ),
            None,
            span,
        )),
    }
}

#[inline(always)]
fn check_binary_equality(
    operator: &TokenKind,
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

    if a.is_ptr_type() && b.is_ptr_type()
        || a.is_mut_ptr_type() && b.is_mut_ptr_type()
        || a.is_mut_ptr_type() && b.is_ptr_type()
        || a.is_ptr_type() && b.is_mut_ptr_type()
    {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!(
            "Logical operation ({} {} {}) is not allowed.",
            a, operator, b
        ),
        None,
        span,
    ))
}

fn check_binary_comparasion(
    operator: &TokenKind,
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
    } else if let (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) = (a, b) {
        return check_binary_comparasion(operator, a_subtype, b_subtype, span);
    } else if let (ThrushType::Mut(a_subtype), any) = (a, b) {
        return check_binary_comparasion(operator, a_subtype, any, span);
    } else if let (any, ThrushType::Mut(b_subtype)) = (a, b) {
        return check_binary_comparasion(operator, any, b_subtype, span);
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!(
            "Logical operation ({} {} {}) is not allowed.",
            a, operator, b
        ),
        None,
        span,
    ))
}

fn check_binary_gate(
    operator: &TokenKind,
    a: &ThrushType,
    b: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if let (ThrushType::Bool, ThrushType::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!(
            "Logical operation ({} {} {}) is not allowed.",
            a, operator, b
        ),
        None,
        span,
    ))
}

fn check_binary_shift(
    operator: &TokenKind,
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
    } else if let (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) = (a, b) {
        return check_binary_shift(operator, a_subtype, b_subtype, span);
    } else if let (ThrushType::Mut(a_subtype), any) = (a, b) {
        return check_binary_shift(operator, a_subtype, any, span);
    } else if let (any, ThrushType::Mut(b_subtype)) = (a, b) {
        return check_binary_shift(operator, any, b_subtype, span);
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!(
            "Arithmetic operation ({} {} {}) is not allowed.",
            a, operator, b
        ),
        None,
        span,
    ))
}

pub fn check_unary(
    operator: &TokenKind,
    a: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match operator {
        TokenKind::Minus | TokenKind::PlusPlus | TokenKind::MinusMinus => {
            check_general_unary(operator, a, span)
        }
        TokenKind::Bang => check_unary_instr_bang(a, span),
        _ => Ok(()),
    }
}

fn check_general_unary(
    operator: &TokenKind,
    a: &ThrushType,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    if a.is_integer_type() || a.is_float_type() || a.is_mut_numeric_type() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!(
            "Arithmetic operation '{}' with '{}' is not allowed.",
            operator, a
        ),
        None,
        span,
    ))
}

fn check_unary_instr_bang(a: &ThrushType, span: Span) -> Result<(), ThrushCompilerIssue> {
    if let ThrushType::Bool = a {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Type checking"),
        format!("Logical operation (!{}) is not allowed.", a),
        None,
        span,
    ))
}

pub fn check_type(
    target_type: &ThrushType,
    from_type: &ThrushType,
    expression: Option<&Instruction>,
    operator: Option<&TokenKind>,
    error: ThrushCompilerIssue,
) -> Result<(), ThrushCompilerIssue> {
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
        (ThrushType::Char, ThrushType::Char, None) => Ok(()),
        (ThrushType::Str, ThrushType::Str, None) => Ok(()),
        (ThrushType::Struct(_, target_fields), ThrushType::Struct(_, from_fields), None) => {
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

        (ThrushType::Me(_), ThrushType::Me(_), None) => Ok(()),

        (ThrushType::Me(_), ThrushType::Struct(_, _), None) => Ok(()),

        (ThrushType::Struct(_, _) | ThrushType::Me(_), ThrushType::Ptr(_), None) => Ok(()),

        (
            target_type,
            ThrushType::Mut(from_type),
            Some(
                TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Bang
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) if !target_type.is_mut_type() => {
            check_type(target_type, from_type, expression, operator, error)?;
            Ok(())
        }

        (
            ThrushType::Mut(target_type),
            any_type,
            Some(
                TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Bang
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) if !any_type.is_mut_type() => {
            check_type(target_type, any_type, expression, operator, error)?;
            Ok(())
        }

        (
            ThrushType::Mut(target_type),
            ThrushType::Mut(from_type),
            Some(
                TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Bang
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => {
            check_type(target_type, from_type, expression, operator, error)?;
            Ok(())
        }

        (ThrushType::Ptr(None), ThrushType::Ptr(None), None) => Ok(()),
        (ThrushType::Ptr(Some(target_type)), ThrushType::Ptr(Some(from_type)), None) => {
            check_type(target_type, from_type, expression, operator, error)?;
            Ok(())
        }

        (ThrushType::Ptr(Some(typed)), any, None) => {
            check_type(typed, any, expression, operator, error)?;
            Ok(())
        }

        (
            ThrushType::Bool,
            ThrushType::Bool,
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
            ThrushType::S8,
            ThrushType::S8 | ThrushType::U8,
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
            ThrushType::S16,
            ThrushType::S16 | ThrushType::S8 | ThrushType::U16 | ThrushType::U8,
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
            ThrushType::S32,
            ThrushType::S32
            | ThrushType::S16
            | ThrushType::S8
            | ThrushType::U32
            | ThrushType::U16
            | ThrushType::U8,
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
            ThrushType::S64,
            ThrushType::S64
            | ThrushType::S32
            | ThrushType::S16
            | ThrushType::S8
            | ThrushType::U64
            | ThrushType::U32
            | ThrushType::U16
            | ThrushType::U8,
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
            ThrushType::U8,
            ThrushType::U8,
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
            ThrushType::U16,
            ThrushType::U16 | ThrushType::U8,
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
            ThrushType::U32,
            ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
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
            ThrushType::U64,
            ThrushType::U64 | ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
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
            ThrushType::F32,
            ThrushType::F32,
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
            ThrushType::F64,
            ThrushType::F64 | ThrushType::F32,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),

        _ => Err(error),
    }
}
