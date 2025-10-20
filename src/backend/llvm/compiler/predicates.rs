use crate::backend::llvm::compiler::abort;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::tokentype::TokenType;

use std::path::PathBuf;

use inkwell::{FloatPredicate, IntPredicate};

#[must_use]
pub fn integer(
    context: &mut LLVMCodeGenContext<'_, '_>,
    operator: &TokenType,
    lhs_signed: bool,
    rhs_signed: bool,
    span: Span,
) -> IntPredicate {
    match operator {
        TokenType::EqEq => IntPredicate::EQ,
        TokenType::BangEq => IntPredicate::NE,

        TokenType::Greater if !lhs_signed && !rhs_signed => IntPredicate::UGT,
        TokenType::Greater if lhs_signed && !rhs_signed => IntPredicate::SGT,
        TokenType::Greater if !lhs_signed && rhs_signed => IntPredicate::SGT,
        TokenType::Greater if lhs_signed && rhs_signed => IntPredicate::SGT,
        TokenType::GreaterEq if !lhs_signed && !rhs_signed => IntPredicate::UGE,
        TokenType::GreaterEq if lhs_signed && !rhs_signed => IntPredicate::SGE,
        TokenType::GreaterEq if !lhs_signed && rhs_signed => IntPredicate::SGE,
        TokenType::GreaterEq if lhs_signed && rhs_signed => IntPredicate::SGE,
        TokenType::Less if !lhs_signed && !rhs_signed => IntPredicate::ULT,
        TokenType::Less if lhs_signed && !rhs_signed => IntPredicate::SLT,
        TokenType::Less if !lhs_signed && rhs_signed => IntPredicate::SLT,
        TokenType::Less if lhs_signed && rhs_signed => IntPredicate::SLT,
        TokenType::LessEq if !lhs_signed && !rhs_signed => IntPredicate::ULE,
        TokenType::LessEq if lhs_signed && !rhs_signed => IntPredicate::SLE,
        TokenType::LessEq if !lhs_signed && rhs_signed => IntPredicate::SLE,
        TokenType::LessEq if lhs_signed && rhs_signed => IntPredicate::SLE,

        what => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as integer predicate!", what),
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

#[must_use]
#[inline]
pub fn pointer(
    context: &mut LLVMCodeGenContext<'_, '_>,
    operator: &TokenType,
    span: Span,
) -> IntPredicate {
    match operator {
        TokenType::EqEq => IntPredicate::EQ,
        TokenType::BangEq => IntPredicate::NE,

        what => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as pointer predicate!", what),
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

#[must_use]
#[inline]
pub fn float(
    context: &mut LLVMCodeGenContext<'_, '_>,
    operator: &TokenType,
    span: Span,
) -> FloatPredicate {
    match operator {
        TokenType::EqEq => FloatPredicate::OEQ,
        TokenType::BangEq => FloatPredicate::ONE,
        TokenType::Greater => FloatPredicate::OGT,
        TokenType::GreaterEq => FloatPredicate::OGE,
        TokenType::Less => FloatPredicate::OLT,
        TokenType::LessEq => FloatPredicate::OLE,

        what => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' as floating-point predicate!", what),
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
