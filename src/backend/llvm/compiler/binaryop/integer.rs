use {
    super::super::{Instruction, symbols::SymbolsTable, unaryop, utils, valuegen},
    crate::{
        backend::llvm::compiler::predicates,
        middle::{
            statement::{BinaryOp, UnaryOp},
            types::{TokenKind, Type},
        },
    },
    inkwell::{
        builder::Builder,
        context::Context,
        values::{BasicValueEnum, IntValue},
    },
    std::cmp::Ordering,
};

pub fn int_operation<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    mut left: IntValue<'ctx>,
    mut right: IntValue<'ctx>,
    signatures: (bool, bool),
    operator: &TokenKind,
) -> BasicValueEnum<'ctx> {
    match operator {
        TokenKind::Plus => builder.build_int_nsw_add(left, right, "").unwrap().into(),
        TokenKind::Minus => builder.build_int_nsw_sub(left, right, "").unwrap().into(),
        TokenKind::Star => builder.build_int_nsw_mul(left, right, "").unwrap().into(),
        TokenKind::Slash if signatures.0 || signatures.1 => builder
            .build_int_signed_div(left, right, "")
            .unwrap()
            .into(),
        TokenKind::Slash if !signatures.0 && !signatures.1 => builder
            .build_int_unsigned_div(left, right, "")
            .unwrap()
            .into(),
        TokenKind::LShift => builder.build_left_shift(left, right, "").unwrap().into(),
        TokenKind::RShift => builder
            .build_right_shift(left, right, signatures.0 || signatures.1, "")
            .unwrap()
            .into(),

        op if op.is_logical_type() => {
            match left
                .get_type()
                .get_bit_width()
                .cmp(&right.get_type().get_bit_width())
            {
                Ordering::Greater => {
                    right = builder
                        .build_int_cast_sign_flag(right, left.get_type(), signatures.0, "")
                        .unwrap();
                }
                Ordering::Less => {
                    left = builder
                        .build_int_cast_sign_flag(left, right.get_type(), signatures.1, "")
                        .unwrap();
                }
                _ => {}
            }

            builder
                .build_int_compare(
                    predicates::integer(operator, signatures.0, signatures.1),
                    left,
                    right,
                    "",
                )
                .unwrap()
                .into()
        }

        op if op.is_logical_gate() => {
            if left.get_type() != context.bool_type() {
                left = builder
                    .build_int_cast_sign_flag(left, context.bool_type(), signatures.0, "")
                    .unwrap()
            }

            if right.get_type() != context.bool_type() {
                right = builder
                    .build_int_cast_sign_flag(right, context.bool_type(), signatures.0, "")
                    .unwrap()
            }

            if let TokenKind::And = op {
                return builder.build_and(left, right, "").unwrap().into();
            }

            if let TokenKind::Or = op {
                return builder.build_or(left, right, "").unwrap().into();
            }

            unreachable!()
        }
        _ => unreachable!(),
    }
}

pub fn integer_binaryop<'ctx>(
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    symbols: &SymbolsTable<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let context: &Context = symbols.get_llvm_context();
    let builder: &Builder = symbols.get_llvm_builder();

    /* ######################################################################


        BOOLEAN BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Boolean(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let left_compiled: IntValue = valuegen::integer(context, left_type, *left as u64, false);
        let right_compiled: IntValue = valuegen::integer(context, right_type, *right as u64, false);

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            binary.1,
        );
    }

    if let (
        Instruction::Boolean(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left as u64, false);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (false, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let mut right_compiled: IntValue =
            valuegen::integer(context, &Type::Bool, *right as u64, false);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (left_type.is_signed_integer_type(), false),
            binary.1,
        );
    }

    if let (
        Instruction::Boolean(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::LocalRef {
            name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left as u64, false);

        let mut right_compiled: BasicValueEnum = symbols.get_allocated_symbol(name).load(symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (false, left_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            name,
            kind: left_type,
            ..
        }
        | Instruction::ConstRef {
            name,
            kind: left_type,
            ..
        },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = symbols.get_allocated_symbol(name).load(symbols);

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right as u64, false);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (left_type.is_signed_integer_type(), false),
            binary.1,
        );
    }

    /* ######################################################################


        CHAR BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Char(_, left, ..),
        TokenKind::BangEq | TokenKind::EqEq,
        Instruction::Char(_, right, ..),
    ) = binary
    {
        let operator: &TokenKind = binary.1;

        let left_compiled: IntValue = valuegen::integer(context, &Type::S8, *left as u64, false);
        let right_compiled: IntValue = valuegen::integer(context, &Type::S8, *right as u64, false);

        return builder
            .build_int_compare(
                predicates::integer(operator, false, false),
                left_compiled,
                right_compiled,
                "",
            )
            .unwrap()
            .into();
    }

    /* ######################################################################


        UNARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            kind: left_call_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, symbols);

        let left_call_type: &Type = left_call_type;

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_call_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed, ..),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            name: left_name,
            kind: left_type,
            ..
        }
        | Instruction::ConstRef {
            name: left_name,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(left_name).load(symbols);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let mut right_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(right_name).load(symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, right_dissasembled, symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::unary_op(builder, context, left_dissasembled, symbols);

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    /* ######################################################################


        CALL - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Call {
            kind: left_call_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, symbols);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_call_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_call_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed, _),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            kind: left_call_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, symbols);

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_call_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            name: left_name,
            kind: left_type,
            ..
        }
        | Instruction::ConstRef {
            name: left_name,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(left_name).load(symbols);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            kind: left_call_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, symbols);

        let mut right_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(right_name).load(symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_call_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            integer_binaryop(left_dissasembled, target_type, symbols);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            kind: left_call_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, symbols);

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            integer_binaryop(right_dissasembled, target_type, symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_call_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    /* ######################################################################


        INTEGER - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Integer(left_type, left_num, left_signed, ..),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);
        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, *right_signed),
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed, ..),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum = symbols.get_allocated_symbol(name).load(symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (*left_signed, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            name: left_name,
            kind: left_type,
            ..
        }
        | Instruction::ConstRef {
            name: left_name,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(left_name).load(symbols);

        let mut right_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(right_name).load(symbols);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        }
        | Instruction::ConstRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let mut right_compiled: BasicValueEnum =
            symbols.get_allocated_symbol(right_name).load(symbols);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(target_type, right_type, right_compiled, builder, context)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed, ..),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            name,
            kind: left_type,
            ..
        }
        | Instruction::ConstRef {
            name,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = symbols.get_allocated_symbol(name).load(symbols);

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(target_type, left_type, left_compiled, builder, context)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled,
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let left_dissasembled: BinaryOp = expression.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed, ..),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            expression,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    /* ######################################################################


        BINARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    /* ######################################################################


        GROUP - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression,
            kind: left_type,
            ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = expression.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            expression,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, symbols).into_int_value();

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, symbols).into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            builder,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    println!("{:#?}", binary);
    unimplemented!()
}
