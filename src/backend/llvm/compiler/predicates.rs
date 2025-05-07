use inkwell::{FloatPredicate, IntPredicate};

use crate::middle::types::TokenKind;

#[must_use]
pub fn integer(operator: &TokenKind, left_signed: bool, right_signed: bool) -> IntPredicate {
    match operator {
        TokenKind::EqEq => IntPredicate::EQ,
        TokenKind::BangEq => IntPredicate::NE,
        TokenKind::Greater if !left_signed && !right_signed => IntPredicate::UGT,
        TokenKind::Greater if left_signed && !right_signed => IntPredicate::SGT,
        TokenKind::Greater if !left_signed && right_signed => IntPredicate::SGT,
        TokenKind::Greater if left_signed && right_signed => IntPredicate::SGT,
        TokenKind::GreaterEq if !left_signed && !right_signed => IntPredicate::UGE,
        TokenKind::GreaterEq if left_signed && !right_signed => IntPredicate::SGE,
        TokenKind::GreaterEq if !left_signed && right_signed => IntPredicate::SGE,
        TokenKind::GreaterEq if left_signed && right_signed => IntPredicate::SGE,
        TokenKind::Less if !left_signed && !right_signed => IntPredicate::ULT,
        TokenKind::Less if left_signed && !right_signed => IntPredicate::SLT,
        TokenKind::Less if !left_signed && right_signed => IntPredicate::SLT,
        TokenKind::Less if left_signed && right_signed => IntPredicate::SLT,
        TokenKind::LessEq if !left_signed && !right_signed => IntPredicate::ULE,
        TokenKind::LessEq if left_signed && !right_signed => IntPredicate::SLE,
        TokenKind::LessEq if !left_signed && right_signed => IntPredicate::SLE,
        TokenKind::LessEq if left_signed && right_signed => IntPredicate::SLE,
        _ => unreachable!(),
    }
}

pub fn pointer(operator: &TokenKind) -> IntPredicate {
    match operator {
        TokenKind::EqEq => IntPredicate::EQ,
        TokenKind::BangEq => IntPredicate::NE,
        _ => unreachable!(),
    }
}

#[must_use]
pub fn float(operator: &TokenKind) -> FloatPredicate {
    // ESTABILIZAR ESTA COSA EN EL FUTURO IGUAL QUE LOS INTEGER PREDICATE (DETERMINAR SI TIENE SIGNO Y CAMBIAR EL PREDICATE A CONVENIR)
    match operator {
        TokenKind::EqEq => FloatPredicate::OEQ,
        TokenKind::BangEq => FloatPredicate::ONE,
        TokenKind::Greater => FloatPredicate::OGT,
        TokenKind::GreaterEq => FloatPredicate::OGE,
        TokenKind::Less => FloatPredicate::OLT,
        TokenKind::LessEq => FloatPredicate::OLE,
        _ => unreachable!(),
    }
}
