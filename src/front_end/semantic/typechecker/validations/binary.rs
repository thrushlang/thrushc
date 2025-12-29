use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::types::Type;

#[inline]
pub fn validate_binary(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match op {
        TokenType::Arith
        | TokenType::Star
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Plus => self::validate_binary_arithmetic(op, a, b, span),
        TokenType::Xor => self::validate_xor(op, a, b, span),
        TokenType::Bor => self::validate_bor(op, a, b, span),
        TokenType::BAnd => self::validate_band(op, a, b, span),
        TokenType::BangEq | TokenType::EqEq => self::validate_binary_equality(op, a, b, span),
        TokenType::LessEq | TokenType::Less | TokenType::GreaterEq | TokenType::Greater => {
            self::validate_binary_comparasion(op, a, b, span)
        }
        TokenType::LShift | TokenType::RShift => self::validate_binary_shift(op, a, b, span),
        TokenType::And | TokenType::Or => self::validate_binary_gate(op, a, b, span),

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
fn validate_band(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid bit operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_bor(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid bit operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_xor(op: &TokenType, a: &Type, b: &Type, span: Span) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid bit operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_binary_gate(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match (a, b) {
        (Type::Bool(..), Type::Bool(..)) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid logical operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_binary_shift(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid arithmetic operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_binary_comparasion(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),
        (Type::FPPC128(..), Type::FPPC128(..)) => Ok(()),
        (Type::FX8680(..), Type::FX8680(..)) => Ok(()),
        (
            Type::F32(..) | Type::F64(..) | Type::F128(..),
            Type::F32(..) | Type::F64(..) | Type::F128(..),
        ) => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid relational operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_binary_equality(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),
        (
            Type::F32(..) | Type::F64(..) | Type::F128(..),
            Type::F32(..) | Type::F64(..) | Type::F128(..),
        ) => Ok(()),
        (Type::Bool(..), Type::Bool(..)) => Ok(()),
        (Type::Char(..), Type::Char(..)) => Ok(()),
        (Type::FX8680(..), Type::FX8680(..)) => Ok(()),
        (Type::FPPC128(..), Type::FPPC128(..)) => Ok(()),

        _ if a.is_ptr_type() && b.is_ptr_type() => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid relational operation.", a, op, b),
            None,
            span,
        )),
    }
}

#[inline]
fn validate_binary_arithmetic(
    op: &TokenType,
    a: &Type,
    b: &Type,
    span: Span,
) -> Result<(), CompilationIssue> {
    match (a, b) {
        (
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..),
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..),
        ) => Ok(()),
        (Type::SSize(..), Type::SSize(..)) => Ok(()),
        (Type::USize(..), Type::USize(..)) => Ok(()),
        (Type::FPPC128(..), Type::FPPC128(..)) => Ok(()),
        (Type::FX8680(..), Type::FX8680(..)) => Ok(()),
        (
            Type::F32(..) | Type::F64(..) | Type::F128(..),
            Type::F32(..) | Type::F64(..) | Type::F128(..),
        ) => Ok(()),
        (Type::Ptr(..), Type::Ptr(..)) if a == b => Ok(()),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0030,
            format!("'{} {} {}' isn't a valid arithmetic operation.", a, op, b),
            None,
            span,
        )),
    }
}
