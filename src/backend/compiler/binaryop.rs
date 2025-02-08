use {
    super::{
        super::super::frontend::lexer::{DataTypes, TokenKind},
        objects::CompilerObjects,
        types::BinaryOp,
        utils, Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        values::{BasicValueEnum, FloatValue, IntValue},
    },
    std::cmp::Ordering,
};

fn build_int_op<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    mut left: IntValue<'ctx>,
    mut right: IntValue<'ctx>,
    signatures: (bool, bool),
    op: TokenKind,
) -> BasicValueEnum<'ctx> {
    match op {
        TokenKind::Plus => builder.build_int_nsw_add(left, right, "").unwrap().into(),
        TokenKind::Minus => builder.build_int_nsw_sub(left, right, "").unwrap().into(),
        TokenKind::Star => builder.build_int_nsw_mul(left, right, "").unwrap().into(),
        TokenKind::Slash => builder
            .build_int_signed_div(left, right, "")
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
                Ordering::Equal => {}
            }

            builder
                .build_int_compare(
                    op.as_int_predicate(signatures.0, signatures.1),
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

            unimplemented!()
        }
        _ => unreachable!(),
    }
}

fn build_float_op<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    mut left: FloatValue<'ctx>,
    mut right: FloatValue<'ctx>,
    op: TokenKind,
) -> BasicValueEnum<'ctx> {
    match op {
        TokenKind::Plus => builder.build_float_add(left, right, "").unwrap().into(),
        TokenKind::Minus => builder.build_float_sub(left, right, "").unwrap().into(),
        TokenKind::Star => builder.build_float_mul(left, right, "").unwrap().into(),
        TokenKind::Slash => builder.build_float_div(left, right, "").unwrap().into(),
        op if op.is_logical_type() => {
            if left.get_type() != context.f64_type() {
                left = builder
                    .build_float_cast(left, context.f64_type(), "")
                    .unwrap()
            }

            if right.get_type() != context.f64_type() {
                right = builder
                    .build_float_cast(right, context.f64_type(), "")
                    .unwrap()
            }

            let result: IntValue<'ctx> = builder
                .build_float_compare(op.as_float_predicate(), left, right, "")
                .unwrap();

            builder.build_bit_cast(result, left.get_type(), "").unwrap()
        }

        op if op.is_logical_gate() => {
            let left_bitcasted: IntValue<'_> = builder
                .build_bit_cast(left, context.i64_type(), "")
                .unwrap()
                .into_int_value();

            let right_bitcasted: IntValue<'_> = builder
                .build_bit_cast(right, context.i64_type(), "")
                .unwrap()
                .into_int_value();

            if let TokenKind::And = op {
                let and_result: IntValue<'_> = builder
                    .build_and(left_bitcasted, right_bitcasted, "")
                    .unwrap();

                return builder
                    .build_bit_cast(and_result, left.get_type(), "")
                    .unwrap();
            }

            if let TokenKind::Or = op {
                let or_result: IntValue<'_> = builder
                    .build_or(left_bitcasted, right_bitcasted, "")
                    .unwrap();

                return builder
                    .build_bit_cast(or_result, left.get_type(), "")
                    .unwrap();
            }

            unimplemented!()
        }

        _ => unreachable!(),
    }
}

pub fn integer_binaryop<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &DataTypes,
    objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        Instruction::Char(left),
        TokenKind::BangEq | TokenKind::EqEq,
        Instruction::Char(right),
    ) = binary
    {
        let left_compiled: IntValue<'_> =
            utils::build_const_integer(context, &DataTypes::I8, *left as u64, false);

        let right_compiled: IntValue<'_> =
            utils::build_const_integer(context, &DataTypes::I8, *right as u64, false);

        return builder
            .build_int_compare(
                binary.1.as_int_predicate(false, false),
                left_compiled,
                right_compiled,
                "",
            )
            .unwrap()
            .into();
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: IntValue<'_> =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: IntValue<'_> =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, *right_signed),
            *binary.1,
        );
    }

    if let (
        Instruction::RefVar {
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::RefVar {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, left_type),
                objects.find_and_get(left_name).unwrap(),
                "",
            )
            .unwrap()
            .into_int_value();

        let mut right_compiled: IntValue<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, right_type),
                objects.find_and_get(right_name).unwrap(),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::RefVar { name, kind, .. },
    ) = binary
    {
        let mut left_compiled: IntValue<'_> =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: IntValue<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, false),
            *binary.1,
        );
    }

    if let (
        Instruction::RefVar { name, kind, .. },
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: IntValue<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_int_value();

        let mut right_compiled: IntValue<'_> =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            kind,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, *right_signed),
            *binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue<'_> =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, false),
            *binary.1,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let mut right_compiled: IntValue<'_> =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, *right_signed),
            *binary.1,
        );
    }

    if let (
        Instruction::Group { instr, kind },
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let mut right_compiled: IntValue<'_> =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            kind,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, *right_signed),
            *binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group { instr, kind },
    ) = binary
    {
        let mut left_compiled: IntValue<'_> =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (*left_signed, false),
            *binary.1,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::RefVar { name, kind, .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let mut right_compiled: IntValue<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
        );
    }

    if let (
        Instruction::Group {
            instr: left_instr,
            kind: left_type,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            instr: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
        );
    }

    if let (
        Instruction::Group {
            instr,
            kind: left_type,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue<'_> =
            integer_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_int_value();

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: IntValue<'_> =
            integer_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_int_value();
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled,
            (false, false),
            *binary.1,
        );
    }

    println!("{:#?}", binary);

    unimplemented!()
}

pub fn float_binaryop<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &DataTypes,
    objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    println!("{:?}", binary);

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::RefVar {
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
        | TokenKind::GreaterEq,
        Instruction::RefVar {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, left_type),
                objects.find_and_get(left_name).unwrap(),
                "",
            )
            .unwrap()
            .into_float_value();

        let mut right_compiled: FloatValue<'_> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, right_type),
                objects.find_and_get(right_name).unwrap(),
                "",
            )
            .unwrap()
            .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::RefVar { name, kind, .. },
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: FloatValue<'_> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::RefVar { name, kind, .. },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_float_value();

        let mut right_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = utils::float_autocast(
            kind,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let mut right_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Group { instr, kind },
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let mut right_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = utils::float_autocast(
            kind,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::Group { instr, kind },
    ) = binary
    {
        let mut left_compiled: FloatValue<'_> =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::RefVar { name, kind, .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let mut right_compiled: FloatValue<'_> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, kind),
                objects.find_and_get(name).unwrap(),
                "",
            )
            .unwrap()
            .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            kind,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Group {
            instr: left_instr,
            kind: left_type,
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            instr: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    if let (
        Instruction::Group {
            instr,
            kind: left_type,
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
        | TokenKind::GreaterEq,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue<'_> =
            float_binaryop(builder, context, left_dissasembled, target_type, objects)
                .into_float_value();

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: FloatValue<'_> =
            float_binaryop(builder, context, right_dissasembled, target_type, objects)
                .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, *binary.1);
    }

    println!("{:#?}", binary);

    unimplemented!()
}
