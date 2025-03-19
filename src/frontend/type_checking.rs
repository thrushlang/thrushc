use super::super::{
    backend::instruction::Instruction,
    error::ThrushCompilerError,
    frontend::lexer::{TokenKind, Type},
};

#[inline(always)]
fn check_binary_arithmetic(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    match (a, b) {
        (
            Type::I8 | Type::I16 | Type::I32 | Type::I64,
            Type::I8 | Type::I16 | Type::I32 | Type::I64,
        ) => Ok(()),
        (Type::F32 | Type::F64, Type::F32 | Type::F64) => Ok(()),

        _ => Err(ThrushCompilerError::Error(
            String::from("Type checking"),
            format!("Arithmetic operation ({} {} {}) is not allowed.", a, op, b),
            location.0,
            Some(location.1),
        )),
    }
}

#[inline(always)]
fn check_binary_equality(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
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

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_comparasion(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    } else if let (Type::F32 | Type::F64, Type::F32 | Type::F64) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_gate(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    if let (Type::Bool, Type::Bool) = (a, b) {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation ({} {} {}) is not allowed.", a, op, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_binary_shift(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    if let (
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
        Type::I8 | Type::I16 | Type::I32 | Type::I64,
    ) = (a, b)
    {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Arithmetic operation ({} {} {}) is not allowed.", a, op, b),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
pub fn check_binary_types(
    op: &TokenKind,
    a: &Type,
    b: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    match op {
        TokenKind::Star | TokenKind::Slash | TokenKind::Minus | TokenKind::Plus => {
            check_binary_arithmetic(op, a, b, location)
        }
        TokenKind::BangEq | TokenKind::EqEq => check_binary_equality(op, a, b, location),
        TokenKind::LessEq | TokenKind::Less | TokenKind::GreaterEq | TokenKind::Greater => {
            check_binary_comparasion(op, a, b, location)
        }
        TokenKind::LShift | TokenKind::RShift => check_binary_shift(op, a, b, location),
        TokenKind::And | TokenKind::Or => check_binary_gate(op, a, b, location),
        _ => Ok(()),
    }
}

/*

UNARY INSTRUCTION

--------------------
OPERATOR OPERATOR
--------------------
*/

#[inline(always)]
fn check_unary(
    op: &TokenKind,
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    if let Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64 = a {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Arithmetic operation '{}' with '{}' is not allowed.", op, a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
fn check_unary_instr_bang(
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    if let Type::Bool = a {
        return Ok(());
    }

    Err(ThrushCompilerError::Error(
        String::from("Type checking"),
        format!("Logical operation (!{}) is not allowed.", a),
        location.0,
        Some(location.1),
    ))
}

#[inline(always)]
pub fn check_unary_types(
    op: &TokenKind,
    a: &Type,
    location: (usize, (usize, usize)),
) -> Result<(), ThrushCompilerError> {
    match op {
        TokenKind::Minus | TokenKind::PlusPlus | TokenKind::MinusMinus => {
            check_unary(op, a, location)
        }
        TokenKind::Bang => check_unary_instr_bang(a, location),
        _ => Ok(()),
    }
}

pub fn check_types(
    target: Type,
    from: Option<Type>,
    value: Option<&Instruction>,
    op: Option<&TokenKind>,
    error: ThrushCompilerError,
) -> Result<(), ThrushCompilerError> {
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
