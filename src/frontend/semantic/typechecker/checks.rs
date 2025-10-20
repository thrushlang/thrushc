use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::metadata::TypeCheckerExprMetadata,
        types::ast::{Ast, metadata::cast::CastMetadata},
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn check_types(
    lhs: &Type,
    rhs: &Type,
    expr: Option<&Ast>,
    op: Option<&TokenType>,
    metadata: TypeCheckerExprMetadata,
) -> Result<(), ThrushCompilerIssue> {
    let span: Span = metadata.get_span();

    let error: ThrushCompilerIssue = ThrushCompilerIssue::Error(
        "Mismatched types".into(),
        format!("Expected '{}' but found '{}'.", lhs, rhs),
        None,
        span,
    );

    if let Some(Ast::BinaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, None, Some(op), metadata);
    }

    if let Some(Ast::UnaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, None, Some(op), metadata);
    }

    if let Some(Ast::Group {
        expression,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, Some(expression), None, metadata);
    }

    match (lhs, rhs, op) {
        (Type::Char, Type::Char, None) => Ok(()),

        (Type::Struct(_, lhs, mod1), Type::Struct(_, rhs, mod2), None) => {
            if lhs.len() != rhs.len() {
                return Err(error);
            }

            if mod1 != mod2 {
                return Err(ThrushCompilerIssue::Error(
                    "Mismatched structure type modificator".into(),
                    format!(
                        "Expected structure type with '{}' attributes but found '{}'.",
                        mod1, mod2
                    ),
                    None,
                    span,
                ));
            }

            lhs.iter()
                .zip(rhs.iter())
                .try_for_each(|(lhs, rhs)| self::check_types(lhs, rhs, None, None, metadata))?;

            Ok(())
        }

        (Type::Addr, Type::Addr, None) => Ok(()),

        (Type::Fn(lhs, ret1, mod1), Type::Fn(rhs, ret2, mod2), None) => {
            if lhs.len() != rhs.len() {
                return Err(error);
            }

            if ret1 != ret2 {
                return Err(error);
            }

            if mod1 != mod2 {
                return Err(ThrushCompilerIssue::Error(
                    "Mismatched function reference type modificator".into(),
                    format!(
                        "Expected function reference type with '{}' attributes but found '{}'.",
                        mod1, mod2
                    ),
                    None,
                    span,
                ));
            }

            lhs.iter()
                .zip(rhs.iter())
                .try_for_each(|(lhs, rhs)| self::check_types(lhs, rhs, None, None, metadata))?;

            Ok(())
        }

        (Type::Const(lhs), Type::Const(rhs), None) => {
            self::check_types(lhs, rhs, None, None, metadata)
        }

        (Type::Const(lhs), rhs, None) => self::check_types(lhs, rhs, None, None, metadata),

        (Type::FixedArray(lhs, lhs_size), Type::FixedArray(rhs, rhs_size), None) => {
            if lhs_size == rhs_size {
                self::check_types(lhs, rhs, None, None, metadata)?;
                return Ok(());
            }

            Err(error)
        }

        (Type::Array(lhs), Type::Array(rhs), None) => {
            self::check_types(lhs, rhs, None, None, metadata)?;
            Ok(())
        }

        (Type::Ptr(None), Type::Ptr(None), Some(TokenType::EqEq | TokenType::BangEq) | None) => {
            Ok(())
        }

        (
            Type::Ptr(Some(lhs)),
            Type::Ptr(Some(rhs)),
            Some(TokenType::EqEq | TokenType::BangEq) | None,
        ) => {
            self::check_types(lhs, rhs, expr, op, metadata)?;
            Ok(())
        }

        (
            Type::Ptr(..) | Type::NullPtr,
            Type::Ptr(..) | Type::NullPtr,
            Some(TokenType::EqEq | TokenType::BangEq) | None,
        ) => Ok(()),

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
            Type::S8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
            )
            | None,
        ) => Ok(()),

        (
            Type::U128,
            Type::U128 | Type::U64 | Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
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
                | TokenType::Arith
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
                | TokenType::Arith
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),

        (
            Type::F128,
            Type::F128 | Type::F64 | Type::F32,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),

        (
            Type::FX8680,
            Type::FX8680,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),

        (
            Type::FPPC128,
            Type::FPPC128,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus,
            )
            | None,
        ) => Ok(()),

        (
            Type::S8,
            Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
            )
            | None,
        ) if metadata.is_literal() => Ok(()),

        (
            Type::S16,
            Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::Arith
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
            )
            | None,
        ) if metadata.is_literal() => Ok(()),

        (
            Type::S32,
            Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::Arith
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
            )
            | None,
        ) if metadata.is_literal() => Ok(()),

        (
            Type::S64,
            Type::U64 | Type::U32 | Type::U16 | Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Arith
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::BAnd,
            )
            | None,
        ) if metadata.is_literal() => Ok(()),

        (Type::Void, Type::Void, None) => Ok(()),

        _ => Err(error),
    }
}

pub fn check_type_cast(
    cast_type: &Type,
    from_type: &Type,
    metadata: &CastMetadata,
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let is_allocated: bool = metadata.is_allocated();

    let abort_cast = || {
        Err(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!("Cannot cast '{}' to '{}'.", from_type, cast_type),
            None,
            *span,
        ))
    };

    if from_type.is_ptr_type() && cast_type.is_integer_type() {
        return Ok(());
    }

    if from_type.is_integer_type() && cast_type.is_integer_type() {
        return Ok(());
    }

    if from_type.is_float_type() && cast_type.is_float_type() {
        return Ok(());
    }

    if from_type.is_ptr_type() && cast_type.is_ptr_type() {
        return Ok(());
    }

    if (from_type.is_float_type()
        || from_type.is_integer_type()
        || from_type.is_struct_type()
        || from_type.is_array_type()
        || from_type.is_fixed_array_type())
        && is_allocated
        && cast_type.is_ptr_type()
    {
        return Ok(());
    }

    if from_type.is_ptr_type() && cast_type.is_const_type() {
        let lhs: &Type = cast_type.get_type_with_depth(1);

        self::check_type_cast(lhs, from_type, metadata, span)?;

        return Ok(());
    }

    abort_cast()
}
