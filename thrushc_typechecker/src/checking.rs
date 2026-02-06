use thrushc_ast::{Ast, metadata::CastingMetadata};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;

use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::metadata::TypeCheckerExpressionMetadata;

pub fn check_types(
    lhs: &Type,
    rhs: &Type,
    expr: Option<&Ast>,
    operator: Option<&TokenType>,
    metadata: TypeCheckerExpressionMetadata,
    span: Span,
) -> Result<(), CompilationIssue> {
    let error: CompilationIssue = CompilationIssue::Error(
        CompilationIssueCode::E0020,
        format!("Expected '{}' type, got '{}' type.", lhs, rhs),
        None,
        span,
    );

    if let Some(Ast::BinaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, None, Some(op), metadata, span);
    }

    if let Some(Ast::UnaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, None, Some(op), metadata, span);
    }

    if let Some(Ast::Group {
        expression,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::check_types(lhs, expression_type, Some(expression), None, metadata, span);
    }

    match (lhs, rhs, operator) {
        (Type::Char(..), Type::Char(..), None) => Ok(()),

        (Type::Struct(_, lhs, mod1, ..), Type::Struct(_, rhs, mod2, ..), None) => {
            if lhs.len() != rhs.len() {
                return Err(error);
            }

            if mod1 != mod2 {
                return Err(CompilationIssue::Error(
                    CompilationIssueCode::E0021,
                    format!(
                        "Expected structure type with '{}' attributes but found '{}'.",
                        mod1, mod2
                    ),
                    None,
                    span,
                ));
            }

            {
                for (lhs, rhs) in lhs.iter().zip(rhs) {
                    self::check_types(lhs, rhs, None, None, metadata, span)?;
                }
            }

            Ok(())
        }

        (Type::Addr(..), Type::Addr(..), None) => Ok(()),

        (Type::Fn(lhs, ret1, mod1, ..), Type::Fn(rhs, ret2, mod2, ..), None) => {
            if lhs.len() != rhs.len() {
                return Err(error);
            }

            if ret1 != ret2 {
                return Err(error);
            }

            if mod1 != mod2 {
                return Err(CompilationIssue::Error(
                    CompilationIssueCode::E0021,
                    format!(
                        "Expected function reference type with '{}' attributes but found '{}'.",
                        mod1, mod2
                    ),
                    None,
                    span,
                ));
            }

            {
                for (lhs, rhs) in lhs.iter().zip(rhs) {
                    self::check_types(lhs, rhs, None, None, metadata, span)?;
                }
            }

            Ok(())
        }

        (Type::Const(lhs, ..), Type::Const(rhs, ..), None) => {
            self::check_types(lhs, rhs, None, None, metadata, span)
        }

        (Type::Const(lhs, ..), rhs, None) => {
            self::check_types(lhs, rhs, None, None, metadata, span)
        }

        (Type::FixedArray(lhs, lhs_size, ..), Type::FixedArray(rhs, rhs_size, ..), None) => {
            if lhs_size == rhs_size {
                self::check_types(lhs, rhs, None, None, metadata, span)?;
                return Ok(());
            }

            Err(error)
        }

        (Type::Array { base_type: lhs, .. }, Type::Array { base_type: rhs, .. }, None) => {
            self::check_types(lhs, rhs, None, None, metadata, span)?;
            Ok(())
        }

        (
            Type::Ptr(None, ..),
            Type::Ptr(None, ..),
            Some(TokenType::EqEq | TokenType::BangEq) | None,
        ) => Ok(()),

        (
            Type::Ptr(Some(lhs), ..),
            Type::Ptr(Some(rhs), ..),
            Some(TokenType::EqEq | TokenType::BangEq) | None,
        ) => {
            self::check_types(lhs, rhs, expr, operator, metadata, span)?;
            Ok(())
        }

        (Type::Ptr(..), Type::Ptr(..), Some(TokenType::EqEq | TokenType::BangEq) | None) => Ok(()),

        (
            Type::Bool(..),
            Type::Bool(..),
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
            Type::S8(..),
            Type::S8(..),
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
            Type::S16(..),
            Type::S16(..) | Type::S8(..),
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
            Type::S32(..),
            Type::S32(..) | Type::S16(..) | Type::S8(..),
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
            Type::S64(..),
            Type::S64(..) | Type::S32(..) | Type::S16(..) | Type::S8(..),
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
            Type::SSize(..),
            Type::SSize(..) | Type::S64(..) | Type::S32(..) | Type::S16(..) | Type::S8(..),
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
            Type::U8(..),
            Type::U8(..),
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
            Type::U16(..),
            Type::U16(..) | Type::U8(..),
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
            Type::U32(..),
            Type::U32(..) | Type::U16(..) | Type::U8(..),
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
            Type::U64(..),
            Type::U64(..) | Type::U32(..) | Type::U16(..) | Type::U8(..),
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
            Type::U128(..),
            Type::U128(..) | Type::U64(..) | Type::U32(..) | Type::U16(..) | Type::U8(..),
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
            Type::USize(..),
            Type::USize(..) | Type::U64(..) | Type::U32(..) | Type::U16(..) | Type::U8(..),
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
            Type::F32(..),
            Type::F32(..),
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
            Type::F64(..),
            Type::F64(..) | Type::F32(..),
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
            Type::F128(..),
            Type::F128(..) | Type::F64(..) | Type::F32(..),
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
            Type::FX8680(..),
            Type::FX8680(..),
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
            Type::FPPC128(..),
            Type::FPPC128(..),
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
            Type::S8(..),
            Type::U8(..),
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
            Type::S16(..),
            Type::U16(..) | Type::U8(..),
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
            Type::S32(..),
            Type::U32(..) | Type::U16(..) | Type::U8(..),
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
            Type::S64(..),
            Type::U64(..) | Type::U32(..) | Type::U16(..) | Type::U8(..),
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

        (Type::Void(..), Type::Void(..), None) => Ok(()),

        _ => Err(error),
    }
}

pub fn check_type_cast(
    cast_type: &Type,
    from_type: &Type,
    metadata: &CastingMetadata,
    span: &Span,
) -> Result<(), CompilationIssue> {
    let is_allocated: bool = metadata.is_allocated();

    match (from_type, cast_type) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::SSize(..)
            | Type::Char(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::SSize(..)
            | Type::Char(..),
        ) => Ok(()),

        (
            Type::F32(..) | Type::F64(..) | Type::F128(..),
            Type::F32(..) | Type::F64(..) | Type::F128(..),
        ) => Ok(()),

        (Type::FX8680(..), Type::FX8680(..)) => Ok(()),
        (Type::FPPC128(..), Type::FPPC128(..)) => Ok(()),

        (
            Type::Ptr(..) | Type::Addr(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::SSize(..),
        ) => Ok(()),

        (Type::Ptr(..) | Type::Addr(..), Type::Ptr(..) | Type::Addr(..)) => Ok(()),
        (Type::Ptr(..), Type::Array { .. }) if is_allocated => Ok(()),
        (Type::Ptr(None, ..), Type::Fn { .. }) if is_allocated => Ok(()),

        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::SSize(..)
            | Type::Char(..)
            | Type::F32(..)
            | Type::F64(..)
            | Type::F128(..)
            | Type::FX8680(..)
            | Type::FPPC128(..)
            | Type::Bool(..)
            | Type::Struct(..)
            | Type::Array { .. }
            | Type::FixedArray(..)
            | Type::Fn(..),
            Type::Ptr(..) | Type::Addr(..),
        ) if is_allocated => Ok(()),

        (Type::Const(inner_type, ..), to) => self::check_type_cast(to, inner_type, metadata, span),

        (Type::Ptr(..) | Type::Addr(..), Type::Const(inner_type, ..)) => {
            self::check_type_cast(inner_type, from_type, metadata, span)
        }

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0032,
            format!(
                "Cannot cast type '{}' to '{}'. Types are incompatible for cast.",
                from_type, cast_type
            ),
            None,
            *span,
        )),
    }
}
