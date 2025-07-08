use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::position::TypeCheckerPosition,
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn check(
    lhs: &Type,
    rhs: &Type,
    expr: Option<&Ast>,
    op: Option<&TokenType>,
    position: Option<TypeCheckerPosition>,
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let error: ThrushCompilerIssue = ThrushCompilerIssue::Error(
        "Mismatched types".into(),
        format!("Expected '{}' but found '{}'.", lhs, rhs),
        None,
        *span,
    );

    if let Some(Ast::BinaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check(lhs, expression_type, None, Some(op), position, span);
    }

    if let Some(Ast::UnaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check(lhs, expression_type, None, Some(op), position, span);
    }

    if let Some(Ast::Group {
        expression,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check(lhs, expression_type, Some(expression), None, position, span);
    }

    match (lhs, rhs, op) {
        (Type::Char, Type::Char, None) => Ok(()),

        (Type::Str, Type::Str, None) => Ok(()),

        (Type::Struct(_, target_fields), Type::Struct(_, from_fields), None) => {
            if target_fields.len() != from_fields.len() {
                return Err(error);
            }

            target_fields.iter().zip(from_fields.iter()).try_for_each(
                |(target_field, from_field)| {
                    self::check(target_field, from_field, None, None, position, span)
                },
            )?;

            Ok(())
        }

        (Type::Addr, Type::Addr, None) => Ok(()),

        (Type::Const(lhs), Type::Const(rhs), None) => {
            self::check(lhs, rhs, None, None, position, span)
        }

        (Type::Const(lhs), rhs, None) => self::check(lhs, rhs, None, None, position, span),

        (Type::FixedArray(type_a, size_a), Type::FixedArray(type_b, size_b), None) => {
            if size_a == size_b {
                self::check(type_a, type_b, None, None, position, span)?;
                return Ok(());
            }

            Err(error)
        }

        (Type::Array(lhs), Type::Array(rhs), None) => {
            self::check(lhs, rhs, None, None, position, span)?;

            Ok(())
        }

        (Type::Mut(lhs), rhs, None)
            if position.is_some_and(|position| position.at_local())
                && !rhs.is_mut_type()
                && !rhs.is_ptr_type() =>
        {
            self::check(lhs, rhs, expr, op, position, span)?;

            Ok(())
        }

        (Type::Mut(..), Type::Mut(..), _)
            if position.is_some_and(|position| position.at_local()) =>
        {
            Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Memory aliasing isn't allowed at high-level pointers.".into(),
                None,
                *span,
            ))
        }

        (Type::Mut(lhs), Type::Mut(rhs), None) => {
            self::check(lhs, rhs, expr, op, position, span)?;

            Ok(())
        }

        (Type::Ptr(None), Type::Ptr(None), None) => Ok(()),

        (Type::Ptr(Some(lhs)), Type::Ptr(Some(rhs)), None) => {
            self::check(lhs, rhs, expr, op, position, span)?;

            Ok(())
        }

        (
            Type::Bool,
            Type::Bool,
            Some(
                TokenType::BangEq
                | TokenType::EqEq
                | TokenType::LessEq
                | TokenType::Less
                | TokenType::Greater
                | TokenType::GreaterEq
                | TokenType::And
                | TokenType::Or
                | TokenType::Bang,
            )
            | None,
        ) => Ok(()),
        (
            Type::S8,
            Type::S8 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::S16,
            Type::S16 | Type::S8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::S32,
            Type::S32 | Type::S16 | Type::S8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::S64,
            Type::S64 | Type::S32 | Type::S16 | Type::S8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::U8,
            Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::U16,
            Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::U32,
            Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::U64,
            Type::U64 | Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::F32,
            Type::F32,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),
        (
            Type::F64,
            Type::F64 | Type::F32,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),

        (Type::Void, Type::Void, None) => Ok(()),

        _ => Err(error),
    }
}
