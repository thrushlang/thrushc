use super::super::super::super::frontend::lexer::{TokenKind, Type};

use super::super::{
    Instruction, call, objects::CompilerObjects, typegen, types::BinaryOp, types::UnaryOp, unaryop,
    utils,
};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, FloatValue},
};

fn build_float_op<'ctx>(
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
            if left.get_type() != right.get_type() {
                left = builder
                    .build_float_cast(left, right.get_type(), "")
                    .unwrap()
            }

            if right.get_type() != left.get_type() {
                right = builder
                    .build_float_cast(right, left.get_type(), "")
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

pub fn float_binaryop<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    binary: BinaryOp<'ctx>,
    target_type: &Type,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    /* ######################################################################


        FLOAT - BINARY EXPRESSIONS


    ########################################################################*/

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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        | TokenKind::BangEq
        | TokenKind::EqEq
        | TokenKind::LessEq
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: UnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_dissasembled.2.get_type(),
            None,
            left_compiled,
            builder,
            context,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = utils::float_autocast(
            target_type,
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
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
        | TokenKind::GreaterEq,
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
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
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

        if let Some(new_left_compiled) = utils::float_autocast(
            target_type,
            left_dissasembled.2.get_type(),
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
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled,
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                typegen::type_int_to_llvm_int_type(context, left_type),
            );

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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

        if let Some(new_right_compiled) = utils::integer_autocast(
            target_type,
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
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
            unaryop::compile_unary_op(builder, context, left_dissasembled, compiler_objects);

        let mut right_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                typegen::type_int_to_llvm_int_type(context, right_type),
            );

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2.get_type(),
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

        return build_float_op(
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
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled,
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOp = left_instr.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        let right_dissasembled: UnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, right_dissasembled, compiler_objects);

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
            right_dissasembled.2.get_type(),
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled,
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
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

        let mut right_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        if let Some(new_left_compiled) = utils::integer_autocast(
            target_type,
            left_dissasembled.2.get_type(),
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
            right_compiled = new_right_compiled.into_float_value();
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled,
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
        | TokenKind::GreaterEq,
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
        | TokenKind::GreaterEq,
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
            compiler_objects,
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
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
        | TokenKind::GreaterEq,
        Instruction::Call {
            name: right_call_name,
            args: right_arguments,
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, left_type),
            );

        let mut right_compiled: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (right_call_name, right_call_type, right_arguments),
            compiler_objects,
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
        | TokenKind::GreaterEq,
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
                typegen::type_float_to_llvm_float_type(context, right_type),
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
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
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
        | TokenKind::GreaterEq,
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

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
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
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    /* ######################################################################


        REFERENCE - BINARY EXPRESSIONS


    ########################################################################*/

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
        | TokenKind::GreaterEq,
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
        let mut left_compiled: FloatValue = compiler_objects
            .get_allocated_object(left_name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, left_type),
            )
            .into_float_value();

        let mut right_compiled: FloatValue = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, right_type),
            )
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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        Instruction::LocalRef {
            name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let mut right_compiled: FloatValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, right_type),
            )
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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let mut left_compiled: FloatValue = compiler_objects
            .get_allocated_object(name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, left_type),
            )
            .into_float_value();

        let mut right_compiled: FloatValue =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        | TokenKind::GreaterEq,
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

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue = compiler_objects
            .get_allocated_object(right_name)
            .load_from_memory(
                builder,
                typegen::type_float_to_llvm_float_type(context, right_type),
            )
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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        )
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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
            compiler_objects,
        );

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        );

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
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            expression: right_instr,
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
            compiler_objects,
        );

        let right_dissasembled: BinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        );

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
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed),
    ) = binary
    {
        let left_dissasembled: BinaryOp = expression.as_binary();

        let mut left_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        )
        .into_float_value();

        let mut right_compiled: FloatValue =
            utils::build_const_float(builder, context, right_type, *right_num, *right_signed);

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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        Instruction::Group {
            expression,
            kind: right_type,
        },
    ) = binary
    {
        let mut left_compiled: FloatValue =
            utils::build_const_float(builder, context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: FloatValue = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        )
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

        return build_float_op(builder, left_compiled, right_compiled, binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOp = expression.as_binary();

        let mut left_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            left_dissasembled,
            target_type,
            compiler_objects,
        );

        let right_dissasembled: BinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        );

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
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
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
            expression,
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
            compiler_objects,
        );

        let right_dissasembled: BinaryOp = expression.as_binary();

        let mut right_compiled: BasicValueEnum = float_binaryop(
            module,
            builder,
            context,
            right_dissasembled,
            target_type,
            compiler_objects,
        );

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
            right_type,
            None,
            right_compiled,
            builder,
            context,
        ) {
            right_compiled = new_right_compiled;
        }

        return build_float_op(
            builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    println!("{:#?}", binary);
    unimplemented!()
}
