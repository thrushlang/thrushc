use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::metadata::TypeCheckerExprMetadata,
        types::ast::{Ast, metadata::cast::CastMetadata},
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn type_check(
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
        return self::type_check(lhs, expression_type, None, Some(op), metadata);
    }

    if let Some(Ast::UnaryOp {
        operator: op,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::type_check(lhs, expression_type, None, Some(op), metadata);
    }

    if let Some(Ast::Group {
        expression,
        kind: expression_type,
        ..
    }) = expr
    {
        return self::type_check(lhs, expression_type, Some(expression), None, metadata);
    }

    match (lhs, rhs, op) {
        (Type::Char, Type::Char, None) => Ok(()),

        (Type::Str, Type::Str, None) => Ok(()),

        (Type::Struct(_, lhs), Type::Struct(_, rhs), None) => {
            if lhs.len() != rhs.len() {
                return Err(error);
            }

            lhs.iter()
                .zip(rhs.iter())
                .try_for_each(|(lhs, rhs)| self::type_check(lhs, rhs, None, None, metadata))?;

            Ok(())
        }

        (Type::Addr, Type::Addr, None) => Ok(()),

        (Type::Const(lhs), Type::Const(rhs), None) => {
            self::type_check(lhs, rhs, None, None, metadata)
        }

        (Type::Const(lhs), rhs, None) => self::type_check(lhs, rhs, None, None, metadata),

        (Type::FixedArray(lhs, lhs_size), Type::FixedArray(rhs, rhs_size), None) => {
            if lhs_size == rhs_size {
                self::type_check(lhs, rhs, None, None, metadata)?;
                return Ok(());
            }

            Err(error)
        }

        (Type::Array(lhs), Type::Array(rhs), None) => {
            self::type_check(lhs, rhs, None, None, metadata)?;

            Ok(())
        }

        (Type::Mut(lhs), rhs, None)
            if metadata
                .get_position()
                .is_some_and(|position| position.at_local())
                && !rhs.is_mut_type()
                && !rhs.is_ptr_type() =>
        {
            self::type_check(lhs, rhs, expr, op, metadata)?;

            Ok(())
        }

        (Type::Mut(..), Type::Mut(..), _)
            if metadata
                .get_position()
                .is_some_and(|position| position.at_local()) =>
        {
            Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Memory aliasing isn't allowed at high-level pointers.".into(),
                None,
                span,
            ))
        }

        (Type::Mut(lhs), Type::Mut(rhs), None) => {
            self::type_check(lhs, rhs, expr, op, metadata)?;

            Ok(())
        }

        (
            Type::Ptr(None),
            Type::Ptr(None),
            Some(
                TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::EqEq
                | TokenType::BangEq,
            )
            | None,
        ) => Ok(()),

        (
            Type::Ptr(Some(lhs)),
            Type::Ptr(Some(rhs)),
            Some(
                TokenType::Xor
                | TokenType::Bor
                | TokenType::Not
                | TokenType::EqEq
                | TokenType::BangEq,
            )
            | None,
        ) => {
            self::type_check(lhs, rhs, expr, op, metadata)?;

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
            Type::S8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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

        (
            Type::S8,
            Type::U8,
            Some(
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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
                | TokenType::LShift
                | TokenType::RShift
                | TokenType::PlusPlus
                | TokenType::MinusMinus
                | TokenType::Xor
                | TokenType::Bor
                | TokenType::Not,
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

    if from_type.is_integer_type() && cast_type.is_integer_type() {
        return Ok(());
    }

    if from_type.is_float_type() && cast_type.is_float_type() {
        return Ok(());
    }

    if from_type.is_str_type() && cast_type.is_ptr_type() {
        return Ok(());
    }

    if (from_type.is_str_type()
        || from_type.is_float_type()
        || from_type.is_integer_type()
        || from_type.is_struct_type()
        || from_type.is_array_type()
        || from_type.is_fixed_array_type())
        && is_allocated
        && cast_type.is_ptr_type()
    {
        return Ok(());
    }

    if from_type.is_mut_type() && cast_type.is_mut_type() {
        let lhs_type: &Type = cast_type.get_type_with_depth(1);
        let rhs_type: &Type = from_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    if from_type.is_mut_type() && cast_type.is_ptr_type() {
        let lhs_type: &Type = cast_type.get_type_with_depth(1);
        let rhs_type: &Type = from_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    if from_type.is_const_type() && cast_type.is_ptr_type() {
        let lhs_type: &Type = cast_type.get_type_with_depth(1);
        let rhs_type: &Type = from_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    if from_type.is_ptr_type() && cast_type.is_ptr_type() {
        let lhs_type: &Type = cast_type.get_type_with_depth(1);
        let rhs_type: &Type = from_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    abort_cast()
}
