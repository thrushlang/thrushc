use {
    super::super::{ThrushStatement, context::LLVMCodeGenContext, unaryop, valuegen},
    crate::{
        backend::llvm::compiler::{cast, predicates, valuegen::ExpressionModificator},
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType,
            types::{
                lexer::ThrushType,
                representations::{BinaryOperation, UnaryOperation},
            },
        },
    },
    inkwell::{
        builder::Builder,
        context::Context,
        values::{BasicValueEnum, IntValue, PointerValue},
    },
    std::cmp::Ordering,
};

pub fn int_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if left.is_int_value() && right.is_int_value() {
        let mut left: IntValue = left.into_int_value();
        let mut right: IntValue = right.into_int_value();

        return match operator {
            TokenType::Plus => llvm_builder
                .build_int_nsw_add(left, right, "")
                .unwrap()
                .into(),
            TokenType::Minus => llvm_builder
                .build_int_nsw_sub(left, right, "")
                .unwrap()
                .into(),
            TokenType::Star => llvm_builder
                .build_int_nsw_mul(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::LShift => llvm_builder
                .build_left_shift(left, right, "")
                .unwrap()
                .into(),
            TokenType::RShift => llvm_builder
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
                        right = llvm_builder
                            .build_int_cast_sign_flag(right, left.get_type(), signatures.0, "")
                            .unwrap();
                    }
                    Ordering::Less => {
                        left = llvm_builder
                            .build_int_cast_sign_flag(left, right.get_type(), signatures.1, "")
                            .unwrap();
                    }
                    _ => {}
                }

                llvm_builder
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
                if left.get_type() != llvm_context.bool_type() {
                    left = llvm_builder
                        .build_int_cast_sign_flag(left, llvm_context.bool_type(), signatures.0, "")
                        .unwrap()
                }

                if right.get_type() != llvm_context.bool_type() {
                    right = llvm_builder
                        .build_int_cast_sign_flag(right, llvm_context.bool_type(), signatures.0, "")
                        .unwrap()
                }

                if let TokenType::And = op {
                    return llvm_builder.build_and(left, right, "").unwrap().into();
                }

                if let TokenType::Or = op {
                    return llvm_builder.build_or(left, right, "").unwrap().into();
                }

                unreachable!()
            }
            _ => unreachable!(),
        };
    }

    if left.is_pointer_value() && right.is_pointer_value() {
        let left: PointerValue = left.into_pointer_value();
        let right: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_type() => llvm_builder
                .build_int_compare(predicates::pointer(operator), left, right, "")
                .unwrap()
                .into(),
            _ => unreachable!(),
        };
    }

    unreachable!()
}

pub fn integer_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    /* ######################################################################


        PROPERTY BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        ThrushStatement::Property { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Property { .. },
    ) = binary
    {
        let left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Property { .. },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Property { .. },
    ) = binary
    {
        let left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            cast,
            ExpressionModificator::new(false, true),
        );

        let right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    /* ######################################################################


        BOOLEAN BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        ThrushStatement::Boolean {
            kind: left_type,
            value: left,
            ..
        },
        TokenType::BangEq | TokenType::EqEq | TokenType::And | TokenType::Or,
        ThrushStatement::Boolean {
            kind: right_type,
            value: right,
            ..
        },
    ) = binary
    {
        let left_compiled: IntValue = valuegen::integer(llvm_context, left_type, *left, false);
        let right_compiled: IntValue = valuegen::integer(llvm_context, right_type, *right, false);

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (false, false),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Boolean {
            kind: left_type,
            value: left,
            ..
        },
        TokenType::BangEq | TokenType::EqEq | TokenType::And | TokenType::Or,
        ThrushStatement::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue = valuegen::integer(llvm_context, left_type, *left, false);

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (false, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        ThrushStatement::UnaryOp {
            kind: left_type, ..
        },
        TokenType::BangEq | TokenType::EqEq | TokenType::And | TokenType::Or,
        ThrushStatement::Boolean {
            kind: right_type,
            value: right,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right, false);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
            (left_type.is_signed_integer_type(), false),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Boolean {
            kind: left_type,
            value: left,
            ..
        },
        TokenType::BangEq | TokenType::EqEq | TokenType::And | TokenType::Or,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue = valuegen::integer(llvm_context, left_type, *left, false);

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (false, left_type.is_signed_integer_type()),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Reference {
            kind: left_type, ..
        },
        TokenType::BangEq | TokenType::EqEq | TokenType::And | TokenType::Or,
        ThrushStatement::Boolean {
            kind: right_type,
            value: right,
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

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right, false);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
            (left_type.is_signed_integer_type(), false),
            binary.1,
        );
    }

    /* ######################################################################


        CHAR BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        ThrushStatement::Char {
            kind: left_type,
            byte: left,
            ..
        },
        TokenType::BangEq | TokenType::EqEq,
        ThrushStatement::Char {
            kind: right_type,
            byte: right,
            ..
        },
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left_compiled: IntValue = valuegen::integer(llvm_context, left_type, *left, false);
        let right_compiled: IntValue = valuegen::integer(llvm_context, right_type, *right, false);

        return llvm_builder
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
        ThrushStatement::UnaryOp {
            kind: left_type, ..
        },
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
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

        let left_call_type: &ThrushType = left_call_type;

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::UnaryOp {
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::UnaryOp {
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let left_dissasembled: UnaryOperation = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
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

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
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
        ThrushStatement::UnaryOp {
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Reference {
            kind: right_type, ..
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::UnaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: UnaryOperation = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::UnaryOp {
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        let mut right_compiled: IntValue =
            self::integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_call_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
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

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Reference {
            name: right_name,
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

        let mut right_compiled: BasicValueEnum =
            context.get_allocated_symbol(right_name).load(context);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_call_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Call {
            kind: right_call_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            self::integer_binaryop(context, left_dissasembled, cast);

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_call_type, right_compiled)
        {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_call_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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
            integer_binaryop(context, right_dissasembled, cast);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled,
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
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);
        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (*left_signed, *right_signed),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Reference {
            name,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);

        let mut right_compiled: BasicValueEnum = context.get_allocated_symbol(name).load(context);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (*left_signed, right_type.is_signed_integer_type()),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Reference {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let mut right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            cast,
            ExpressionModificator::new(false, true),
        );

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled,
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
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

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            integer_binaryop(context, right_dissasembled, cast);

        if let Some(new_right_compiled) = cast::integer(context, cast, right_type, right_compiled) {
            right_compiled = new_right_compiled;
        }

        return int_operation(
            context,
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
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, right_type, cast, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (*left_signed, right_type.is_signed_integer_type()),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Reference {
            name,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum = context.get_allocated_symbol(name).load(context);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) = cast::integer(context, cast, left_type, left_compiled) {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled,
            right_compiled.into(),
            (left_type.is_signed_integer_type(), *right_signed),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Integer {
            kind: right_type,
            value: right_num,
            signed: right_signed,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = expression.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num, *right_signed);

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (left_type.is_signed_integer_type(), *right_signed),
            binary.1,
        );
    }

    if let (
        ThrushStatement::Integer {
            kind: left_type,
            value: left_num,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Group {
            expression,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left_num, *left_signed);

        let right_dissasembled: BinaryOperation = expression.as_binary();

        let mut right_compiled: IntValue =
            self::integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (*left_signed, right_type.is_signed_integer_type()),
            binary.1,
        );
    }

    /* ######################################################################


        BINARY - BINARY EXPRESSIONS


    ########################################################################*/

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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Group {
            expression: right_instr,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = left_instr.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: BinaryOperation = right_instr.as_binary();

        let mut right_compiled: IntValue =
            self::integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::BinaryOp {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = expression.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: BinaryOperation = binary.2.as_binary();

        let mut right_compiled: IntValue =
            self::integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        ThrushStatement::Group {
            expression,
            kind: right_type,
            ..
        },
    ) = binary
    {
        let left_dissasembled: BinaryOperation = binary.0.as_binary();

        let mut left_compiled: IntValue =
            self::integer_binaryop(context, left_dissasembled, cast).into_int_value();

        let right_dissasembled: BinaryOperation = expression.as_binary();

        let mut right_compiled: IntValue =
            self::integer_binaryop(context, right_dissasembled, cast).into_int_value();

        if let Some(new_left_compiled) =
            cast::integer(context, cast, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            cast::integer(context, cast, right_type, right_compiled.into())
        {
            right_compiled = new_right_compiled.into_int_value();
        }

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
            (
                left_type.is_signed_integer_type(),
                right_type.is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a integer binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
