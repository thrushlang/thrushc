use {
    super::super::{Instruction, context::LLVMCodeGenContext, unaryop, utils, valuegen},
    crate::{
        backend::llvm::compiler::predicates,
        middle::types::{
            backend::llvm::types::{LLVMBinaryOp, LLVMUnaryOp},
            frontend::lexer::{tokenkind::TokenKind, types::ThrushType},
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
    operator: &TokenKind,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if left.is_int_value() && right.is_int_value() {
        let mut left: IntValue = left.into_int_value();
        let mut right: IntValue = right.into_int_value();

        return match operator {
            TokenKind::Plus => llvm_builder
                .build_int_nsw_add(left, right, "")
                .unwrap()
                .into(),
            TokenKind::Minus => llvm_builder
                .build_int_nsw_sub(left, right, "")
                .unwrap()
                .into(),
            TokenKind::Star => llvm_builder
                .build_int_nsw_mul(left, right, "")
                .unwrap()
                .into(),
            TokenKind::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(left, right, "")
                .unwrap()
                .into(),
            TokenKind::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(left, right, "")
                .unwrap()
                .into(),
            TokenKind::LShift => llvm_builder
                .build_left_shift(left, right, "")
                .unwrap()
                .into(),
            TokenKind::RShift => llvm_builder
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

                if let TokenKind::And = op {
                    return llvm_builder.build_and(left, right, "").unwrap().into();
                }

                if let TokenKind::Or = op {
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
    binary: LLVMBinaryOp<'ctx>,
    target_type: &ThrushType,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    /* ######################################################################


        PROPERTY BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Property { .. },
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
        Instruction::Property { .. },
    ) = binary
    {
        let left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, context);

        let right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                binary.0.get_type().is_signed_integer_type(),
                binary.2.get_type().is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    if let (
        Instruction::Property { .. },
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
        Instruction::Property { .. },
    ) = binary
    {
        let left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, context);

        let right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        return int_operation(
            context,
            left_compiled,
            right_compiled,
            (
                binary.0.get_type().is_signed_integer_type(),
                binary.2.get_type().is_signed_integer_type(),
            ),
            binary.1,
        );
    }

    /* ######################################################################


        BOOLEAN BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        Instruction::Boolean(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left as u64, false);
        let right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right as u64, false);

        return int_operation(
            context,
            left_compiled.into(),
            right_compiled.into(),
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
            valuegen::integer(llvm_context, left_type, *left as u64, false);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        Instruction::UnaryOp {
            kind: left_type, ..
        },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right as u64, false);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        Instruction::Boolean(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::LocalRef {
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left as u64, false);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        Instruction::LocalRef {
            kind: left_type, ..
        }
        | Instruction::ConstRef {
            kind: left_type, ..
        },
        TokenKind::BangEq | TokenKind::EqEq | TokenKind::And | TokenKind::Or,
        Instruction::Boolean(right_type, right, ..),
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, context);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right as u64, false);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        Instruction::Char(left_type, left, ..),
        TokenKind::BangEq | TokenKind::EqEq,
        Instruction::Char(right_type, right, ..),
    ) = binary
    {
        let operator: &TokenKind = binary.1;

        let left_compiled: IntValue =
            valuegen::integer(llvm_context, left_type, *left as u64, false);
        let right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right as u64, false);

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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
            valuegen::generate_expression(binary.0, target_type, context);

        let left_call_type: &ThrushType = left_call_type;

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_call_type, right_compiled)
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
            valuegen::generate_expression(binary.0, target_type, context);

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMUnaryOp = binary.2.as_unaryop();

        let mut right_compiled: BasicValueEnum = unaryop::unary_op(context, right_dissasembled);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMUnaryOp = binary.0.as_unaryop();

        let mut left_compiled: BasicValueEnum = unaryop::unary_op(context, left_dissasembled);

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
            valuegen::generate_expression(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_call_type, right_compiled)
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_call_type, right_compiled)
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
            valuegen::generate_expression(binary.0, target_type, context);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
            valuegen::generate_expression(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_call_type, right_compiled)
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
            valuegen::generate_expression(binary.0, target_type, context);

        let mut right_compiled: BasicValueEnum =
            context.get_allocated_symbol(right_name).load(context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: BasicValueEnum =
            integer_binaryop(left_dissasembled, target_type, context);

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_call_type, right_compiled)
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
            valuegen::generate_expression(binary.0, target_type, context);

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: BasicValueEnum =
            integer_binaryop(right_dissasembled, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_call_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);
        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);

        let mut right_compiled: BasicValueEnum = context.get_allocated_symbol(name).load(context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        | TokenKind::GreaterEq
        | TokenKind::LShift
        | TokenKind::RShift
        | TokenKind::And
        | TokenKind::Or,
        Instruction::LocalRef {
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, context);
        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
            kind: right_type, ..
        }
        | Instruction::ConstRef {
            kind: right_type, ..
        },
    ) = binary
    {
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let mut right_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.2, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
        let mut left_compiled: BasicValueEnum =
            valuegen::generate_expression(binary.0, target_type, context);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: BasicValueEnum =
            integer_binaryop(right_dissasembled, target_type, context);

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled)
        {
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, right_type, target_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let mut left_compiled: BasicValueEnum = context.get_allocated_symbol(name).load(context);

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled)
        {
            left_compiled = new_left_compiled;
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let mut right_compiled: IntValue =
            valuegen::integer(llvm_context, right_type, *right_num as u64, *right_signed);

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
            valuegen::integer(llvm_context, left_type, *left_num as u64, *left_signed);

        let right_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = left_instr.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMBinaryOp = right_instr.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMBinaryOp = binary.2.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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
        let left_dissasembled: LLVMBinaryOp = binary.0.as_binary();

        let mut left_compiled: IntValue =
            integer_binaryop(left_dissasembled, target_type, context).into_int_value();

        let right_dissasembled: LLVMBinaryOp = expression.as_binary();

        let mut right_compiled: IntValue =
            integer_binaryop(right_dissasembled, target_type, context).into_int_value();

        if let Some(new_left_compiled) =
            utils::integer_autocast(context, target_type, left_type, left_compiled.into())
        {
            left_compiled = new_left_compiled.into_int_value();
        }

        if let Some(new_right_compiled) =
            utils::integer_autocast(context, target_type, right_type, right_compiled.into())
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

    println!("{:#?}", binary);
    unimplemented!()
}
