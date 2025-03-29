use super::super::{
    backend::compiler::instruction::Instruction,
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
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64,
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
    if matches!(
        (a, b),
        (
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64,
        ) | (Type::F32 | Type::F64, Type::F32 | Type::F64)
            | (Type::Bool, Type::Bool)
            | (Type::Char, Type::Char)
    ) {
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
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
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
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
        Type::S8 | Type::S16 | Type::S32 | Type::S64 | Type::U8 | Type::U16 | Type::U32 | Type::U64,
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
    if a.is_integer_type() || a.is_float_type() {
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
    target_type: Type,
    from_type: Option<Type>,
    expression: Option<&Instruction>,
    op: Option<&TokenKind>,
    error: ThrushCompilerError,
) -> Result<(), ThrushCompilerError> {
    if let Some(Instruction::BinaryOp {
        op,
        kind: expression_type,
        ..
    }) = expression
    {
        return check_types(target_type, Some(*expression_type), None, Some(op), error);
    }

    if let Some(Instruction::UnaryOp {
        op,
        kind: expression_type,
        ..
    }) = expression
    {
        return check_types(target_type, Some(*expression_type), None, Some(op), error);
    }

    if let Some(Instruction::Group {
        instr: expression, ..
    }) = expression
    {
        return check_types(target_type, None, Some(expression), None, error);
    }

    match (target_type, from_type.unwrap(), op) {
        (Type::Char, Type::Char, None) => Ok(()),
        (Type::Str, Type::Str, None) => Ok(()),
        (Type::Struct, Type::Struct | Type::T, None) => Ok(()),
        (Type::T, Type::T, None) => Ok(()),
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
            Type::S8,
            Type::S8 | Type::U8,
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
            Type::S16,
            Type::S16 | Type::S8 | Type::U16 | Type::U8,
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
            Type::S32,
            Type::S32 | Type::S16 | Type::S8 | Type::U32 | Type::U16 | Type::U8,
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
            Type::S64,
            Type::S64
            | Type::S32
            | Type::S16
            | Type::S8
            | Type::U64
            | Type::U32
            | Type::U16
            | Type::U8,
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
            Type::U8,
            Type::U8,
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
            Type::U16,
            Type::U16 | Type::U8,
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
            Type::U32,
            Type::U32 | Type::U16 | Type::U8,
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
            Type::U64,
            Type::U64 | Type::U32 | Type::U16 | Type::U8,
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
