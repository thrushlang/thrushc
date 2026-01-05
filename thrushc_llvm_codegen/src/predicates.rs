use inkwell::FloatPredicate;
use inkwell::IntPredicate;

use thrushc_span::Span;
use thrushc_token::tokentype::TokenType;

use crate::abort;
use crate::context::LLVMCodeGenContext;

#[must_use]
#[inline]
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

        _ => abort::abort_codegen(
            context,
            "Failed to determinate integer predicate!",
            span,
            std::path::PathBuf::from(file!()),
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

        _ => abort::abort_codegen(
            context,
            "Failed to determinate pointer predicate!",
            span,
            std::path::PathBuf::from(file!()),
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

        _ => abort::abort_codegen(
            context,
            "Failed to determinate floating-point predicate!",
            span,
            std::path::PathBuf::from(file!()),
            line!(),
        ),
    }
}
