use crate::{
    backend::llvm::compiler::{predicates, valuegen::ExpressionModificator},
    core::console::logging::{self, LoggingType},
    frontend::{
        lexer::tokentype::TokenType,
        types::{
            lexer::ThrushType,
            representations::{BinaryOperation, UnaryOperation},
        },
    },
};

use super::super::{ThrushStatement, cast, context::LLVMCodeGenContext, unaryop, valuegen};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue},
};

pub fn float_operation<'ctx>(
    builder: &Builder<'ctx>,
    mut left: FloatValue<'ctx>,
    mut right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    match operator {
        TokenType::Plus => builder.build_float_add(left, right, "").unwrap().into(),
        TokenType::Minus => builder.build_float_sub(left, right, "").unwrap().into(),
        TokenType::Star => builder.build_float_mul(left, right, "").unwrap().into(),
        TokenType::Slash => builder.build_float_div(left, right, "").unwrap().into(),
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
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &ThrushType,
) -> BasicValueEnum<'ctx> {
    /* ######################################################################


        FLOAT - BINARY EXPRESSIONS


    ########################################################################*/

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let (
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Float {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_number,
            *left_signed,
        );

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    /* ######################################################################


        UNARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        ThrushStatement::UnaryOp { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(
            context,
            cast,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::Call {
            kind: left_call_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, cast, left_call_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::UnaryOp { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(
            context,
            cast,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_call_type, right_compiled)
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
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_number,
            *left_signed,
        );

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::UnaryOp { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Reference {
            name: right_name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum =
            context.get_allocated_symbol(right_name).load(context);

        if let Some(new_left_compiled) = cast::float(
            context,
            cast,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::BinaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            self::float_binaryop(context, left_dissasembled, cast).into_float_value();

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::UnaryOp { .. },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: FloatValue =
            self::float_binaryop(context, left_dissasembled, cast).into_float_value();

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(
            context,
            cast,
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
        ThrushStatement::UnaryOp { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: BinaryOperation = right_instr.as_binary();

        let mut right_compiled: FloatValue =
            self::float_binaryop(context, right_dissasembled, cast).into_float_value();

        if let Some(new_left_compiled) = cast::float(
            context,
            cast,
            left_dissasembled.2.get_type_unwrapped(),
            left_compiled,
        ) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
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
        ThrushStatement::Call {
            kind: left_call_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_call_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_call_type, right_compiled)
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
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_number,
            *left_signed,
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_call_type, right_compiled)
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
        ThrushStatement::Call {
            kind: left_call_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Float {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_call_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
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
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_call_type, right_compiled)
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
        ThrushStatement::Call {
            kind: left_call_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_call_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::float_binaryop(context, left_dissasembled, cast);

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_call_type, right_compiled)
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
        ThrushStatement::Call {
            kind: left_call_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let right_dissasembled: BinaryOperation = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::float(context, cast, left_call_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_number,
            *left_signed,
        );

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Float {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
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
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::BinaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            self::float_binaryop(context, left_dissasembled, cast).into_float_value();

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            left_type,
            *left_number,
            *left_signed,
        );

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: FloatValue =
            self::float_binaryop(context, right_dissasembled, cast).into_float_value();

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        ThrushStatement::BinaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Float {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: FloatValue =
            self::float_binaryop(context, left_dissasembled, cast).into_float_value();

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        ThrushStatement::BinaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::float_binaryop(context, left_dissasembled, cast);

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Group {
            expression: left_instr,
            kind: left_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::float_binaryop(context, left_dissasembled, cast);

        let right_dissasembled: BinaryOperation = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::Group {
            expression,
            kind: left_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Float {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = expression.as_binary();

        let mut left_compiled: FloatValue =
            self::float_binaryop(context, left_dissasembled, cast).into_float_value();

        let mut right_compiled: FloatValue = valuegen::float(
            llvm_builder,
            llvm_context,
            right_type,
            *right_num,
            *right_signed,
        );

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        ThrushStatement::Float {
            kind: left_type,
            value: left_number,
            signed: left_signed,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Group {
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
            *left_number,
            *left_signed,
        );

        let right_dissasembled: BinaryOperation = expression.as_binary();

        let mut right_compiled: FloatValue =
            self::float_binaryop(context, right_dissasembled, cast).into_float_value();

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_float_value();
        }

        if let Some(new_right_compiled) =
            cast::float(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_float_value();
        }

        return float_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    if let (
        ThrushStatement::Group {
            expression,
            kind: left_type,
            ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = expression.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::float_binaryop(context, left_dissasembled, cast);

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
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
        ThrushStatement::BinaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ThrushStatement::Group {
            expression,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::float_binaryop(context, left_dissasembled, cast);

        let right_dissasembled: BinaryOperation = expression.as_binary();

        let mut right_compiled: BasicValueEnum =
            self::float_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::float(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::float(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return float_operation(
            llvm_builder,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a float binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
