use super::super::{
    backend::instruction::Instruction,
    error::ThrushError,
    frontend::lexer::{TokenKind, Type},
};

#[inline(always)]
fn check_binary_instr_add(
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    match (a, b) {
        (
            Type::I8 | Type::I16 | Type::I32 | Type::I64,
            Type::I8 | Type::I16 | Type::I32 | Type::I64,
        ) => Ok(()),
        (Type::F32 | Type::F64, Type::F32 | Type::F64) => Ok(()),

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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
        return Ok(());
    } else if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    } else if let (Type::Char, Type::Char) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
        return Ok(());
    } else if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    } else if let (Type::Char, Type::Char) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Bool,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Bool,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::Bool | Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Bool,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (Type::Bool, Type::Bool) = (a, b) {
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
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (Type::Bool, Type::Bool) = (a, b) {
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
fn check_binary_instr_shift(
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
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
    a: &Type,
    b: &Type,
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
        TokenKind::LShift | TokenKind::RShift => check_binary_instr_shift(a, b, location),
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
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64 = a {
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
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64 = a {
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
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushError> {
    if let Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64 = a {
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
fn check_unary_instr_bang(a: &Type, location: (usize, (usize, usize))) -> Result<(), ThrushError> {
    if let Type::Bool = a {
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
    a: &Type,
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
    target: Type,
    from: Option<Type>,
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
        (Type::Char, Type::Char, None) => Ok(()),
        (Type::Str, Type::Str, None) => Ok(()),
        (Type::Struct, Type::Struct | Type::Ptr, None) => Ok(()),
        (Type::Ptr, Type::Ptr | Type::Str, None) => Ok(()),
        (
            Type::Generic,
            Type::Generic
            | Type::Ptr
            | Type::Str
            | Type::Char
            | Type::I8
            | Type::I32
            | Type::I64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Struct,
            None,
        ) => Ok(()),
        (
            Type::Bool,
            Type::Bool,
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
            Type::I8,
            Type::I8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::I16,
            Type::I16 | Type::I8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::I32,
            Type::I32 | Type::I16 | Type::I8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::I64,
            Type::I64 | Type::I32 | Type::I16 | Type::I8,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::F32,
            Type::F32,
            Some(
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Star
                | TokenKind::LShift
                | TokenKind::RShift,
            )
            | None,
        ) => Ok(()),
        (
            Type::F64,
            Type::F64 | Type::F32,
            Some(TokenKind::Plus | TokenKind::Minus | TokenKind::Slash | TokenKind::Star) | None,
        ) => Ok(()),

        _ => Err(error),
    }
}
