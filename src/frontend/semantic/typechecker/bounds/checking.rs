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
    target_type: &Type,
    from_type: &Type,
    expression: Option<&Ast>,
    operator: Option<&TokenType>,
    position: Option<TypeCheckerPosition>,
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let error: ThrushCompilerIssue = ThrushCompilerIssue::Error(
        String::from("Mismatched types"),
        format!("Expected '{}' but found '{}'.", target_type, from_type),
        None,
        *span,
    );

    if let Some(Ast::BinaryOp {
        operator,
        kind: expression_type,
        ..
    }) = expression
    {
        return check(
            target_type,
            expression_type,
            None,
            Some(operator),
            position,
            span,
        );
    }

    if let Some(Ast::UnaryOp {
        operator,
        kind: expression_type,
        ..
    }) = expression
    {
        return check(
            target_type,
            expression_type,
            None,
            Some(operator),
            position,
            span,
        );
    }

    if let Some(Ast::Group {
        expression,
        kind: expression_type,
        ..
    }) = expression
    {
        return check(
            target_type,
            expression_type,
            Some(expression),
            None,
            position,
            span,
        );
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
                        check(target_field, from_field, None, None, position, span)
                    },
                )?;

                Ok(())
            }

            (Type::Addr, Type::Addr, None) => Ok(()),

            (
                Type::FixedArray(type_a, size_a),
                Type::FixedArray(type_b, size_b),
                None,
            ) => {
                if size_a == size_b {
                    check(type_a, type_b, None, None, position, span)?;
                    return Ok(());
                }

                Err(error)
            }

            (Type::Array(target_type), Type::Array(from_type), None) => {
                check(target_type, from_type, None, None, position, span)?;

                Ok(())
            }

            (
                Type::Mut(target_type),
                from_type,
                Some(
                    TokenType::BangEq
                    | TokenType::EqEq
                    | TokenType::LessEq
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::GreaterEq
                    | TokenType::And
                    | TokenType::Or
                    | TokenType::Bang
                    | TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift,
                )
                | None,
            ) if position.is_some_and(|position| position.at_local())
                && !from_type.is_mut_type()
                && !from_type.is_ptr_type() =>
            {
                check(target_type, from_type, expression, operator, position, span)?;

                Ok(())
            }

            (
                Type::Mut(..),
                Type::Mut(..),
                _
            ) if position.is_some_and(|position| position.at_local())  => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Memory aliasing isn't allowed at high-level pointers; use Low Level Instructions (LLI) instead.".into(),
                None,
                *span,
            )),

            (
                Type::Mut(target_type),
                Type::Mut(from_type),
                Some(
                    TokenType::BangEq
                    | TokenType::EqEq
                    | TokenType::LessEq
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::GreaterEq
                    | TokenType::And
                    | TokenType::Or
                    | TokenType::Bang
                    | TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift,
                )
                | None,
            ) => {
                check(target_type, from_type, expression, operator, position, span)?;

                Ok(())
            }

            (Type::Ptr(None), Type::Ptr(None), None) => Ok(()),
            (Type::Ptr(Some(target_type)), Type::Ptr(Some(from_type)), None) => {
                check(target_type, from_type, expression, operator, position, span)?;

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
                Type::S16 | Type::U16 | Type::U8 | Type::S8,
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
                Type::S32
                | Type::U32
                | Type::S16
                | Type::U16
                | Type::S8
                | Type::U8,
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
                Type::S64
                | Type::U64
                | Type::S32
                | Type::U32
                | Type::S16
                | Type::U16
                | Type::S8
                | Type::U8,
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
