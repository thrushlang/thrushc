use crate::backends::classical::llvm::compiler::{
    self, abort, codegen, constgen, context::LLVMCodeGenContext, predicates,
};
use crate::frontends::classical::{
    lexer::{span::Span, tokentype::TokenType},
    types::parser::repr::BinaryOperation,
    typesystem::types::Type,
};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};
use std::path::PathBuf;

pub fn bool_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder<'_> = context.get_llvm_builder();

    let (lhs_signed, rhs_signed) = signatures;

    if lhs.is_int_value() && rhs.is_int_value() {
        let (lhs, rhs) = compiler::generation::cast::integer_together(
            context,
            lhs.into_int_value(),
            rhs.into_int_value(),
        );

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(context, operator, lhs_signed, rhs_signed, span),
                    lhs,
                    rhs,
                    "",
                )
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile comparison!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            op if op.is_logical_gate() => match op {
                TokenType::And => llvm_builder
                    .build_and(lhs, rhs, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '&&' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into(),
                TokenType::Or => llvm_builder
                    .build_or(lhs, rhs, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '||' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into(),
                _ => abort::abort_codegen(
                    context,
                    "Failed to compile without a valid operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                ),
            },
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    if lhs.is_float_value() && rhs.is_float_value() {
        let (lhs, rhs) = compiler::generation::cast::float_together(
            context,
            lhs.into_float_value(),
            rhs.into_float_value(),
        );

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_float_compare(predicates::float(context, operator, span), lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile comparison!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    if lhs.is_pointer_value() && rhs.is_pointer_value() {
        let lhs: PointerValue<'_> = lhs.into_pointer_value();
        let rhs: PointerValue<'_> = rhs.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(predicates::pointer(context, operator, span), lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile comparison!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::And
        | TokenType::Or,
        ..,
    ) = binary
    {
        let operator: &'ctx TokenType = binary.1;

        let lhs: BasicValueEnum<'_> = codegen::compile(context, binary.0, cast_type);
        let rhs: BasicValueEnum<'_> = codegen::compile(context, binary.2, cast_type);

        let lhs_type: &Type = binary.0.llvm_get_type(context);
        let rhs_type: &Type = binary.2.llvm_get_type(context);

        return bool_operation(
            context,
            lhs,
            rhs,
            operator,
            (
                lhs_type.is_signed_integer_type(),
                rhs_type.is_signed_integer_type(),
            ),
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn const_bool_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let (lhs_signed, rhs_signed) = signatures;

    if lhs.is_int_value() && rhs.is_int_value() {
        let (lhs, rhs) = compiler::generation::cast::const_integer_together(
            lhs.into_int_value(),
            rhs.into_int_value(),
            signatures,
        );

        return match operator {
            op if op.is_logical_operator() => lhs
                .const_int_compare(
                    predicates::integer(context, operator, lhs_signed, rhs_signed, span),
                    rhs,
                )
                .into(),
            op if op.is_logical_gate() => match op {
                TokenType::And => lhs.const_and(rhs).into(),
                TokenType::Or => lhs.const_or(rhs).into(),
                _ => abort::abort_codegen(
                    context,
                    "Failed to compile without a valid logical operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                ),
            },
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    if lhs.is_float_value() && rhs.is_float_value() {
        let (lhs, rhs) = compiler::generation::cast::const_float_together(
            lhs.into_float_value(),
            rhs.into_float_value(),
        );

        return match operator {
            op if op.is_logical_operator() => lhs
                .const_compare(predicates::float(context, operator, span), rhs)
                .into(),
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    if lhs.is_pointer_value() && rhs.is_pointer_value() {
        let lhs = lhs.into_pointer_value();
        let rhs = rhs.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => match op {
                TokenType::EqEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() == rhs.is_null()) as u64, false)
                    .into(),
                TokenType::BangEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() != rhs.is_null()) as u64, false)
                    .into(),
                _ => abort::abort_codegen(
                    context,
                    "Failed to compile a valid logical operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                ),
            },
            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::And
        | TokenType::Or,
        ..,
    ) = binary
    {
        let operator: &'ctx TokenType = binary.1;

        let lhs: BasicValueEnum<'_> = constgen::compile(context, binary.0, cast_type);
        let rhs: BasicValueEnum<'_> = constgen::compile(context, binary.2, cast_type);

        let lhs_type: &Type = binary.0.llvm_get_type(context);
        let rhs_type: &Type = binary.2.llvm_get_type(context);

        return const_bool_operation(
            context,
            lhs,
            rhs,
            operator,
            (
                lhs_type.is_signed_integer_type(),
                rhs_type.is_signed_integer_type(),
            ),
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}
