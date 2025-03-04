use super::super::{
    backend::instruction::Instruction,
    error::ThrushError,
    frontend::lexer::{DataTypes, TokenKind},
};

#[inline(always)]
fn check_binary_instr_add(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    match (a, b) {
        (
            DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
            DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        ) => Ok(()),
        (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) => Ok(()),

        _ => Err(ThrushError::Error(
            String::from("Type Checking"),
            format!("Arithmatic addition ({} + {}) is not allowed.", a, b),
            location.0,
            Some(location.1),
        )),
    }
}

#[inline(always)]
fn check_binary_instr_sub(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Arithmatic subtraction ({} - {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_div(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Arithmatic division ({} / {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_mul(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Arithmatic multiplication ({} * {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_eqeq(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    } else if let (DataTypes::Bool, DataTypes::Bool) = (a, b) {
        return Ok(());
    } else if let (DataTypes::Char, DataTypes::Char) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} == {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_bangeq(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    } else if let (DataTypes::Bool, DataTypes::Bool) = (a, b) {
        return Ok(());
    } else if let (DataTypes::Char, DataTypes::Char) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} != {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_greater(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 | DataTypes::Bool,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 | DataTypes::Bool,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} > {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_greatereq(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} >= {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_less(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::Bool | DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 | DataTypes::Bool,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} < {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_lesseq(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (DataTypes::F32 | DataTypes::F64, DataTypes::F32 | DataTypes::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} <= {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_and(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (DataTypes::Bool, DataTypes::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} && {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_instr_or(
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (DataTypes::Bool, DataTypes::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation ({} || {}) is not allowed.", a, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
pub fn check_binary_instr(
    op: &TokenKind,
    a: &DataTypes,
    b: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    match op {
        TokenKind::Plus => check_binary_instr_add(a, b, location),
        TokenKind::Minus => check_binary_instr_sub(a, b, location),
        TokenKind::Slash => check_binary_instr_div(a, b, location),
        TokenKind::Star => check_binary_instr_mul(a, b, location),
        TokenKind::EqEq => check_binary_instr_eqeq(a, b, location),
        TokenKind::BangEq => check_binary_instr_bangeq(a, b, location),
        TokenKind::Greater => check_binary_instr_greater(a, b, location),
        TokenKind::GreaterEq => check_binary_instr_greatereq(a, b, location),
        TokenKind::Less => check_binary_instr_less(a, b, location),
        TokenKind::LessEq => check_binary_instr_lesseq(a, b, location),
        TokenKind::And => check_binary_instr_and(a, b, location),
        TokenKind::Or => check_binary_instr_or(a, b, location),
        _ => Ok(()),
    }
}

/*

UNARY INSTRUCTION

--------------------
OPERATOR B OPERATOR
--------------------
*/

#[inline(always)]
fn check_unary_instr_negate(
    a: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let DataTypes::I8
    | DataTypes::I16
    | DataTypes::I32
    | DataTypes::I64
    | DataTypes::F32
    | DataTypes::F64 = a
    {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Negative operation (-{}) is not allowed.", a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_unary_instr_minusminus(
    a: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let DataTypes::I8
    | DataTypes::I16
    | DataTypes::I32
    | DataTypes::I64
    | DataTypes::F32
    | DataTypes::F64 = a
    {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Subtractive operation ({}--) is not allowed.", a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_unary_instr_plusplus(
    a: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let DataTypes::I8
    | DataTypes::I16
    | DataTypes::I32
    | DataTypes::I64
    | DataTypes::F32
    | DataTypes::F64 = a
    {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Additive operation ({}++) is not allowed.", a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_unary_instr_bang(
    a: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let DataTypes::Bool = a {
        return Ok(());
    }

    Err(ThrushError::Error(
        String::from("Type Checking"),
        format!("Logical operation (!{}) is not allowed.", a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
pub fn check_unary_instr(
    op: &TokenKind,
    a: &DataTypes,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    match op {
        TokenKind::PlusPlus => check_unary_instr_plusplus(a, location),
        TokenKind::MinusMinus => check_unary_instr_minusminus(a, location),
        TokenKind::Minus => check_unary_instr_negate(a, location),
        TokenKind::Bang => check_unary_instr_bang(a, location),
        _ => Ok(()),
    }
}

pub fn check_types(
    target: DataTypes,
    from: Option<DataTypes>,
    value: Option<&Instruction>,
    op: Option<&TokenKind>,
    error: ThrushError,
) -> Result<(), ThrushError> {
    if let Some(Instruction::BinaryOp { op, kind, .. }) = value {
        return check_types(target, Some(*kind), None, Some(op), error);
    }

    if let Some(Instruction::UnaryOp { op, kind, .. }) = value {
        return check_types(target, Some(*kind), None, Some(op), error);
    }

    if let Some(Instruction::Group { instr, .. }) = value {
        return check_types(target, None, Some(instr), None, error);
    }

    match (target, from.unwrap(), op) {
        (DataTypes::Char, DataTypes::Char, None) => Ok(()),
        (DataTypes::Str, DataTypes::Str, None) => Ok(()),
        (DataTypes::Struct, DataTypes::Struct | DataTypes::Ptr, None) => Ok(()),
        (DataTypes::Ptr, DataTypes::Ptr | DataTypes::Str, None) => Ok(()),
        (
            DataTypes::Bool,
            DataTypes::Bool,
            Some(
                TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Bang,
            )
            | None,
        ) => Ok(()),
        (
            DataTypes::I8,
            DataTypes::I8,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),
        (
            DataTypes::I16,
            DataTypes::I16 | DataTypes::I8,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),
        (
            DataTypes::I32,
            DataTypes::I32 | DataTypes::I16 | DataTypes::I8,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),
        (
            DataTypes::I64,
            DataTypes::I64 | DataTypes::I32 | DataTypes::I16 | DataTypes::I8,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),
        (
            DataTypes::F32,
            DataTypes::F32,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),
        (
            DataTypes::F64,
            DataTypes::F64 | DataTypes::F32,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),

        _ => Err(error),
    }
}
