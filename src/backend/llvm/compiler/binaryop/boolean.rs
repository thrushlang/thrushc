use {
    super::{
        float::float_binaryop,
        integer::{int_operation, integer_binaryop},
    },
    crate::{
        backend::llvm::compiler::symbols::SymbolsTable,
        middle::{
            instruction::Instruction,
            statement::BinaryOp,
            types::{TokenKind, Type},
        },
    },
    inkwell::{builder::Builder, context::Context, module::Module, values::BasicValueEnum},
};

pub fn bool_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    symbols: &SymbolsTable<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
    ) = binary
    {
        if binary.2.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.2.get_type().is_integer_type() || binary.2.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }
    }

    if let (
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(..) | Instruction::Float(..) | Instruction::Boolean(..),
    ) = binary
    {
        if binary.2.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.2.get_type().is_integer_type() || binary.2.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }
    }

    if let (
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call { .. },
    ) = binary
    {
        if binary.2.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.2.get_type().is_integer_type() || binary.2.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }
    }

    if let (
        Instruction::Call { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. } | Instruction::ConstRef { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, symbols);
        } else if binary.0.get_type().is_integer_type() || binary.0.get_type().is_bool_type() {
            return integer_binaryop(module, builder, context, binary, target_type, symbols);
        }

        unreachable!()
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                symbols,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                symbols,
            );

            return int_operation(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, symbols);
    }

    if let (Instruction::Group { .. }, TokenKind::And | TokenKind::Or, Instruction::Group { .. }) =
        binary
    {
        if binary.0.get_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                symbols,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                symbols,
            );

            return int_operation(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, symbols);
    }

    if let (
        Instruction::Group { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                symbols,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                symbols,
            );

            return int_operation(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, symbols);
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::Group { .. },
    ) = binary
    {
        if binary.0.get_type().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                symbols,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                symbols,
            );

            return int_operation(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, symbols);
    }

    println!("{:#?}", binary);
    unimplemented!()
}
