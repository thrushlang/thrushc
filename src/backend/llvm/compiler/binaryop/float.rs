use crate::{
    backend::llvm::compiler::predicates,
    middle::types::{
        backend::llvm::types::{LLVMBinaryOp, LLVMUnaryOp},
        frontend::lexer::{tokenkind::TokenKind, types::ThrushType},
    },
};

use super::super::{Instruction, cast, context::LLVMCodeGenContext, unaryop, valuegen};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue},
};

pub fn float_operation<'ctx>(
    builder: &Builder<'ctx>,
    mut left: FloatValue<'ctx>,
    mut right: FloatValue<'ctx>,
    operator: &TokenKind,
) -> BasicValueEnum<'ctx> {
    match operator {
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
                .build_float_compare(predicates::float(operator), left, right, "")
                .unwrap()
                .into()
        }

        _ => unreachable!(),
    }
}

pub fn float_binaryop<'ctx>(
    binary: LLVMBinaryOp<'ctx>,
    target_type: &ThrushType,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    /* ######################################################################


        FLOAT - BINARY EXPRESSIONS


    ########################################################################*/

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
        Instruction::Float(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(
            context,
            target_type,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) = cast::float(
            context,
            target_type,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled,
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            kind: left_type, ..
        }
        | Instruction::ConstRef {
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
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum =
            context.get_allocated_symbol(right_name).load(context);

        if let Some(new_left_compiled) = cast::float(
            context,
            target_type,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            float_binaryop(left_dissasembled, target_type, context).into_float_value();

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled,
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: FloatValue =
            float_binaryop(left_dissasembled, target_type, context).into_float_value();

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            target_type,
            right_dissasembled.2.get_type_unwrapped(),
            right_compiled,
        ) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: FloatValue =
            float_binaryop(right_dissasembled, target_type, context).into_float_value();

        if let Some(new_left_compiled) = cast::float(
            context,
            target_type,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(
            llvm_builder,
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
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled,
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled,
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            kind: left_type, ..
        }
        | Instruction::ConstRef {
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
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::LocalRef {
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            float_binaryop(left_dissasembled, target_type, context);

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
            kind: left_type, ..
        }
        | Instruction::ConstRef {
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
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);
        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled,
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::LocalRef {
            kind: left_type, ..
        }
        | Instruction::ConstRef {
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
        Instruction::Float(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled,
            binary.1,
        );
    }

    /* ######################################################################


        BINARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::LocalRef {
            kind: left_type, ..
        }
        | Instruction::ConstRef {
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
        let mut left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
        Instruction::LocalRef {
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            float_binaryop(left_dissasembled, target_type, context).into_float_value();

        let mut right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled,
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: FloatValue =
            float_binaryop(right_dissasembled, target_type, context).into_float_value();

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
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
        Instruction::Float(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            float_binaryop(left_dissasembled, target_type, context).into_float_value();

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: BasicValueEnum =
            float_binaryop(left_dissasembled, target_type, context);

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            float_binaryop(left_dissasembled, target_type, context);

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
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
        | TokenKind::GreaterEq,
        Instruction::Float(right_type, right_num, right_signed, ..),
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut left_compiled: FloatValue =
            float_binaryop(left_dissasembled, target_type, context).into_float_value();

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        Instruction::Float(left_type, left_num, left_signed, ..),
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
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_num,
            *left_signed,
        );

        let right_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut right_compiled: FloatValue =
            float_binaryop(right_dissasembled, target_type, context).into_float_value();

        if let Some(new_left_compiled) =
            cast::float(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
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
        | TokenKind::GreaterEq,
        Instruction::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut left_compiled: BasicValueEnum =
            float_binaryop(left_dissasembled, target_type, context);

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
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
            ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: BasicValueEnum =
            float_binaryop(left_dissasembled, target_type, context);

        let right_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut right_compiled: BasicValueEnum =
            float_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) = cast::float(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, target_type, right_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    println!("{:#?}", binary);
    unimplemented!()
}
