use inkwell::{FloatPredicate, IntPredicate};

use crate::frontend::lexer::tokentype::TokenType;

#[must_use]
pub fn integer(operator: &TokenType, left_signed: bool, right_signed: bool) -> IntPredicate {
    match operator {
        TokenType::EqEq => IntPredicate::EQ,
        TokenType::BangEq => IntPredicate::NE,
        TokenType::Greater if !left_signed && !right_signed => IntPredicate::UGT,
        TokenType::Greater if left_signed && !right_signed => IntPredicate::SGT,
        TokenType::Greater if !left_signed && right_signed => IntPredicate::SGT,
        TokenType::Greater if left_signed && right_signed => IntPredicate::SGT,
        TokenType::GreaterEq if !left_signed && !right_signed => IntPredicate::UGE,
        TokenType::GreaterEq if left_signed && !right_signed => IntPredicate::SGE,
        TokenType::GreaterEq if !left_signed && right_signed => IntPredicate::SGE,
        TokenType::GreaterEq if left_signed && right_signed => IntPredicate::SGE,
        TokenType::Less if !left_signed && !right_signed => IntPredicate::ULT,
        TokenType::Less if left_signed && !right_signed => IntPredicate::SLT,
        TokenType::Less if !left_signed && right_signed => IntPredicate::SLT,
        TokenType::Less if left_signed && right_signed => IntPredicate::SLT,
        TokenType::LessEq if !left_signed && !right_signed => IntPredicate::ULE,
        TokenType::LessEq if left_signed && !right_signed => IntPredicate::SLE,
        TokenType::LessEq if !left_signed && right_signed => IntPredicate::SLE,
        TokenType::LessEq if left_signed && right_signed => IntPredicate::SLE,
        _ => unreachable!(),
    }
}

#[must_use]
pub fn pointer(operator: &TokenType) -> IntPredicate {
    match operator {
        TokenType::EqEq => IntPredicate::EQ,
        TokenType::BangEq => IntPredicate::NE,
        _ => unreachable!(),
    }
}

#[must_use]
pub fn float(operator: &TokenType) -> FloatPredicate {
    match operator {
        TokenType::EqEq => FloatPredicate::OEQ,
        TokenType::BangEq => FloatPredicate::ONE,
        TokenType::Greater => FloatPredicate::OGT,
        TokenType::GreaterEq => FloatPredicate::OGE,
        TokenType::Less => FloatPredicate::OLT,
        TokenType::LessEq => FloatPredicate::OLE,
        _ => unreachable!(),
    }
}
