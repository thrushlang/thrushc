use {
    super::super::{
        super::super::frontend::lexer::{TokenKind, Type},
        Instruction, call,
        objects::CompilerObjects,
        types::{BinaryOp, UnaryOp},
        unaryop, utils,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicValueEnum, IntValue},
    },
    std::cmp::Ordering,
};

pub fn build_int_op<'ctx>(
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

            unreachable!()
        }
        _ => unreachable!(),
    }
}

pub fn compile_integer_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    /* ######################################################################


        BOOLEAN BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Boolean(left),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right),
    ) = binary
    {
        let left_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *left as u64, false);

        let right_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *right as u64, false);

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
        Instruction::Boolean(left),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *left as u64, false);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            target_type,
            right_dissasembled.2,
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
            (false, right_dissasembled.2.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp { .. },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right),
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *right as u64, false);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
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
            left_compiled.into_int_value(),
            right_compiled,
            (left_dissasembled.2.is_signed_integer_type(), false),
            binary.1,
        );
    }

    if let (
        Instruction::Boolean(left),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::LocalRef { name, kind, .. },
    ) = binary
    {
        let localref_type: &Type = kind.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *left as u64, false);

        let mut right_compiled: IntValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, localref_type),
            )
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
            localref_type,
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
            (false, localref_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef { name, kind, .. },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right),
    ) = binary
    {
        let localref_type: &Type = kind.get_type();

        let mut left_compiled: IntValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, localref_type),
            )
            .into_int_value();

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, &Type::Bool, *right as u64, false);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            localref_type,
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
            (localref_type.is_signed_integer_type(), false),
            binary.1,
        );
    }

    /* ######################################################################


        CHAR BINARY EXPRESSIONS


    ########################################################################*/

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

    /* ######################################################################


        UNARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::UnaryOp { .. },
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_dissasembled.2,
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
            (
                left_dissasembled.2.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            compiler_objects,
        )
        .unwrap();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2,
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
            (
                left_call_type.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp { .. },
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
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
        )
        .unwrap();
        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
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
            (
                left_dissasembled.2.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2,
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
            (
                left_type.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp { .. },
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
        let right_type: &Type = right_type.get_type();

        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let mut right_compiled: IntValue =
            utils::build_const_integer(context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
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
            (
                left_dissasembled.2.is_signed_integer_type(),
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, left_type),
            );

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2,
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
            (
                left_type.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp { .. },
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
        let right_type: &Type = right_type.get_type();

        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let mut right_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, right_type),
            );

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
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
            (
                left_dissasembled.2.is_signed_integer_type(),
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2,
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
            (
                left_type.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression: left_instr,
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2,
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
            (
                left_type.is_signed_integer_type(),
                right_dissasembled.2.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::UnaryOp { .. },
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
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2,
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
            (
                left_dissasembled.2.is_signed_integer_type(),
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
            compiler_objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
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
            (
                left_call_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
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
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        let right_type: &Type = right_type.get_type();

        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            compiler_objects,
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
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, left_type),
            );

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
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
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        let right_type: &Type = right_type.get_type();

        let mut left_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (left_call_name, left_call_type, left_arguments),
            compiler_objects,
        )
        .unwrap();

        let mut right_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, right_type),
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

        let mut left_compiled: BasicValueEnum = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        );

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
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
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
            expression: right_instr,
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
            compiler_objects,
        )
        .unwrap();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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
        let left_type: &Type = left_type.get_type();
        let right_type: &Type = right_type.get_type();

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
        let left_type: &Type = left_type.get_type();
        let right_type: &Type = kind.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: IntValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, right_type),
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
            (*left_signed, right_type.is_signed_integer_type()),
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
        let left_type: &Type = left_type.get_type();
        let right_type: &Type = right_type.get_type();

        let mut left_compiled: IntValue = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, left_type),
            )
            .into_int_value();

        let mut right_compiled: IntValue = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, right_type),
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
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let right_type: &Type = right_type.get_type();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let mut right_compiled: IntValue = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, right_type),
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
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let right_type: &Type = right_type.get_type();

        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
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
            (left_type.is_signed_integer_type(), *right_signed),
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
        let left_type: &Type = kind.get_type();
        let right_type: &Type = right_type.get_type();

        let mut left_compiled: IntValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                utils::type_int_to_llvm_int_type(context, left_type),
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
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        Instruction::Group {
            expression,
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
        Instruction::Integer(right_type, right_num, right_signed),
    ) = binary
    {
        let right_type: &Type = right_type.get_type();

        let left_dissasembled: BinaryOp = expression.as_binary();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
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
            (left_type.is_signed_integer_type(), *right_signed),
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
        Instruction::Group {
            expression,
            kind: right_type,
        },
    ) = binary
    {
        let left_type: &Type = left_type.get_type();

        let mut left_compiled: IntValue =
            utils::build_const_integer(context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_int_value();

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: IntValue = compile_integer_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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
