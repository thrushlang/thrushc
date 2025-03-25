use {
    super::{
        super::super::frontend::lexer::{TokenKind, Type},
        Instruction, call,
        objects::CompilerObjects,
        types::BinaryOp,
        utils,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
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
    op: &TokenKind,
) -> BasicValueEnum<'ctx> {
    match op {
        TokenKind::Plus => builder.build_int_nsw_add(left, right, "").unwrap().into(),
        TokenKind::Minus => builder.build_int_nsw_sub(left, right, "").unwrap().into(),
        TokenKind::Star => builder.build_int_nsw_mul(left, right, "").unwrap().into(),
        TokenKind::Slash => builder
            .build_int_signed_div(left, right, "")
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
    op: &TokenKind,
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

            builder
                .build_float_compare(op.as_float_predicate(), left, right, "")
                .unwrap()
                .into()
        }

        _ => unreachable!(),
    }
}

fn build_bool_op<'ctx>(
    builder: &Builder<'ctx>,
    left: IntValue<'ctx>,
    right: IntValue<'ctx>,
    op: &TokenKind,
) -> BasicValueEnum<'ctx> {
    match op {
        op if op.is_logical_type() => builder
            .build_int_compare(op.as_int_predicate(false, false), left, right, "")
            .unwrap()
            .into(),

        op if op.is_logical_gate() => {
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

pub fn integer_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        Instruction::Boolean(left),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And,
        Instruction::Boolean(right),
    ) = binary
    {
        let left_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *left as u64, false);

        let right_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *right as u64, false);

        return build_bool_op(builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::Char(left),
        TokenKind::BangEq | TokenKind::EqEq,
        Instruction::Char(right),
    ) = binary
    {
        let left_compiled: IntValue =
            utils::build_const_integer(context, &Type::S8, *left as u64, false);

        let right_compiled: IntValue =
            utils::build_const_integer(context, &Type::S8, *right as u64, false);

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
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled,
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            left_compiled.into_int_value(),
            right_compiled,
            (false, false),
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
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
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, left_type),
                objects.get_local(left_name),
                "",
            )
            .unwrap();

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, right_type),
                objects.get_local(right_name),
                "",
            )
            .unwrap();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
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
        },
    ) = binary
    {
        let mut left_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, left_type),
                objects.get_local(left_name),
                "",
            )
            .unwrap()
            .into_int_value();

        let mut right_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, right_type),
                objects.get_local(right_name),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { name, kind, .. },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, kind),
                objects.get_local(name),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            kind,
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
        | TokenKind::LShift
        | TokenKind::RShift,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let mut right_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, right_type),
                objects.get_local(right_name),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::LShift
        | TokenKind::RShift,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
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
        | TokenKind::LShift
        | TokenKind::RShift,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef { name, kind, .. },
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
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, kind),
                objects.get_local(name),
                "",
            )
            .unwrap()
            .into_int_value();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            kind,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::Boolean(left),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::LocalRef { name, kind, .. },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *left as u64, false);

        let mut right_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, kind),
                objects.get_local(name),
                "",
            )
            .unwrap()
            .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            &Type::Bool,
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
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef { name, kind, .. },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right),
    ) = binary
    {
        let mut left_compiled: IntValue = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, kind),
                objects.get_local(name),
                "",
            )
            .unwrap()
            .into_int_value();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *right as u64, false);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            kind,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            &Type::Bool,
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
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            instr,
            kind: left_type,
        },
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    if let (
        Instruction::Integer(left_type, left_num, left_signed),
        TokenKind::Plus
        | TokenKind::Slash
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::LShift
        | TokenKind::RShift,
        Instruction::Group {
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
            instr: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_int_op(
            context,
            builder,
            left_compiled.into_int_value(),
            right_compiled.into_int_value(),
            (false, false),
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
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

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
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
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: IntValue = integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_type,
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
            binary.1,
        );
    }

    println!("{:#?}", binary);

    unimplemented!()
}

pub fn float_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
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
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: FloatValue =
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
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
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled,
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: FloatValue =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled,
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
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
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, left_type),
                objects.get_local(left_name),
                "",
            )
            .unwrap();

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, right_type),
                objects.get_local(right_name),
                "",
            )
            .unwrap();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
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
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, left_type),
                objects.get_local(left_name),
                "",
            )
            .unwrap()
            .into_float_value();

        let mut right_compiled: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, right_type),
                objects.get_local(right_name),
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
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
        Instruction::LocalRef { name, kind, .. },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, kind),
                objects.get_local(name),
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::LocalRef { name, kind, .. },
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
        let mut left_compiled: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, kind),
                objects.get_local(name),
                "",
            )
            .unwrap()
            .into_float_value();

        let mut right_compiled: FloatValue =
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::LocalRef {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, right_type),
                objects.get_local(right_name),
                "",
            )
            .unwrap()
            .into_float_value();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_type,
            None,
            left_compiled.into(),
            builder,
            context,
        ) {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_type,
            None,
            right_compiled.into(),
            builder,
            context,
        ) {
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::BinaryOp {
            kind: left_type, ..
        },
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue =
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
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

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
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
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            objects,
        )
        .unwrap();

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_call_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Call {
            name: left_call_name,
            args: left_arguments,
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
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Group {
            instr: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            objects,
        )
        .unwrap();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_call_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            instr: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            instr,
            kind: left_type,
        },
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = instr.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue =
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed),
        TokenKind::Plus | TokenKind::Slash | TokenKind::Minus | TokenKind::Star,
        Instruction::Group {
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        )
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

        return build_float_op(context, builder, left_compiled, right_compiled, binary.1);
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

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            instr,
            kind: right_type,
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            objects,
        );

        let right_dissasembled: BinaryOp = instr.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            objects,
        );

        if let Some(new_left_compiled) = utils::float_autocast(
            left_type,
            target_type,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            right_type,
            target_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            context,
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    println!("{:#?}", binary);

    unimplemented!()
}

pub fn bool_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
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
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }

        unreachable!()
    }

    if let (
        Instruction::LocalRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }

        unreachable!()
    }

    if let (
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
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
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
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
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.2.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.2.get_data_type().is_integer_type()
            || binary.2.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }
    }

    if let (
        Instruction::LocalRef { .. },
        TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq
        | TokenKind::And
        | TokenKind::Or,
        Instruction::Integer(_, _, _) | Instruction::Float(_, _, _) | Instruction::Boolean(_),
    ) = binary
    {
        if binary.2.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.2.get_data_type().is_integer_type()
            || binary.2.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }
    }

    if let (
        Instruction::LocalRef { .. },
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
        if binary.2.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.2.get_data_type().is_integer_type()
            || binary.2.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
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
        Instruction::LocalRef { .. },
    ) = binary
    {
        if binary.0.get_data_type().is_float_type() {
            return float_binaryop(module, builder, context, binary, target_type, objects);
        } else if binary.0.get_data_type().is_integer_type()
            || binary.0.get_data_type().is_bool_type()
        {
            return integer_binaryop(module, builder, context, binary, target_type, objects);
        }

        unreachable!()
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_data_type_recursive().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, objects);
    }

    if let (Instruction::Group { .. }, TokenKind::And | TokenKind::Or, Instruction::Group { .. }) =
        binary
    {
        if binary.0.get_data_type_recursive().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, objects);
    }

    if let (
        Instruction::Group { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::BinaryOp { .. },
    ) = binary
    {
        if binary.0.get_data_type_recursive().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, objects);
    }

    if let (
        Instruction::BinaryOp { .. },
        TokenKind::And | TokenKind::Or,
        Instruction::Group { .. },
    ) = binary
    {
        if binary.0.get_data_type_recursive().is_float_type() {
            let left_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.0.as_binary(),
                target_type,
                objects,
            );

            let right_compiled: BasicValueEnum = float_binaryop(
                module,
                builder,
                context,
                binary.2.as_binary(),
                target_type,
                objects,
            );

            return build_int_op(
                context,
                builder,
                left_compiled.into_int_value(),
                right_compiled.into_int_value(),
                (false, false),
                binary.1,
            );
        }

        return integer_binaryop(module, builder, context, binary, target_type, objects);
    }

    println!("{:#?}", binary);
    unimplemented!()
}
