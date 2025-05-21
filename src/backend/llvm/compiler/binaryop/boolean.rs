use {
    super::{
        float::float_binaryop,
        integer::{int_operation, integer_binaryop},
    },
    crate::{
        backend::llvm::compiler::{binaryop::ptr::ptr_binaryop, context::LLVMCodeGenContext},
        middle::types::{
            backend::llvm::types::LLVMBinaryOp,
            frontend::{
                lexer::{tokenkind::TokenKind, types::ThrushType},
                parser::stmts::stmt::ThrushStatement,
            },
        },
    },
    inkwell::values::BasicValueEnum,
};

pub fn bool_binaryop<'ctx>(
    binary: LLVMBinaryOp<'ctx>,
    target_type_unwrapped: &ThrushType,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Call { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. },
    ) = binary
    {
        if binary.2.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.2.get_type_unwrapped().is_integer_type()
            || binary.2.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }
    }

    if let (
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. }
        | ThrushStatement::NullPtr { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Call { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        } else if binary.2.get_type_unwrapped().is_mut_ptr_type()
            || binary.2.get_type_unwrapped().is_ptr_type()
        {
            return ptr_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Integer { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Boolean { .. }
        | ThrushStatement::NullPtr { .. },
    ) = binary
    {
        if binary.2.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.2.get_type_unwrapped().is_integer_type()
            || binary.2.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        } else if binary.2.get_type_unwrapped().is_mut_ptr_type()
            || binary.2.get_type_unwrapped().is_ptr_type()
        {
            return ptr_binaryop(binary, target_type_unwrapped, context);
        }
    }

    if let (
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::Call { .. },
    ) = binary
    {
        if binary.2.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.2.get_type_unwrapped().is_integer_type()
            || binary.2.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }
    }

    if let (
        ThrushStatement::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        ThrushStatement::LocalRef { .. } | ThrushStatement::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            return float_binaryop(binary, target_type_unwrapped, context);
        } else if binary.0.get_type_unwrapped().is_integer_type()
            || binary.0.get_type_unwrapped().is_bool_type()
        {
            return integer_binaryop(binary, target_type_unwrapped, context);
        }

        unreachable!()
    }

    if let (
        ThrushStatement::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        ThrushStatement::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            let left_compiled: BasicValueEnum =
                float_binaryop(binary.0.as_binary(), target_type_unwrapped, context);

            let right_compiled: BasicValueEnum =
                float_binaryop(binary.2.as_binary(), target_type_unwrapped, context);

            return int_operation(
                context,
                left_compiled,
                right_compiled,
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(binary, target_type_unwrapped, context);
    }

    if let (
        ThrushStatement::Group { .. },
        TokenKind::And | TokenKind::Or,
        ThrushStatement::Group { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            let left_compiled: BasicValueEnum =
                float_binaryop(binary.0.as_binary(), target_type_unwrapped, context);

            let right_compiled: BasicValueEnum =
                float_binaryop(binary.2.as_binary(), target_type_unwrapped, context);

            return int_operation(
                context,
                left_compiled,
                right_compiled,
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(binary, target_type_unwrapped, context);
    }

    if let (
        ThrushStatement::Group { .. },
        TokenKind::And | TokenKind::Or,
        ThrushStatement::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            let left_compiled: BasicValueEnum =
                float_binaryop(binary.0.as_binary(), target_type_unwrapped, context);

            let right_compiled: BasicValueEnum =
                float_binaryop(binary.2.as_binary(), target_type_unwrapped, context);

            return int_operation(
                context,
                left_compiled,
                right_compiled,
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(binary, target_type_unwrapped, context);
    }

    if let (
        ThrushStatement::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        ThrushStatement::Group { .. },
    ) = binary
    {
        if binary.0.get_type_unwrapped().is_float_type() {
            let left_compiled: BasicValueEnum =
                float_binaryop(binary.0.as_binary(), target_type_unwrapped, context);

            let right_compiled: BasicValueEnum =
                float_binaryop(binary.2.as_binary(), target_type_unwrapped, context);

            return int_operation(
                context,
                left_compiled,
                right_compiled,
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(binary, target_type_unwrapped, context);
    }

    println!("{:#?}", binary);
    unimplemented!()
}
