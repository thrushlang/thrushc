use crate::abort;
use crate::cast;
use crate::codegen;
use crate::context::LLVMCodeGenContext;
use crate::predicates;

use thrushc_entities::BinaryOperation;
use thrushc_span::Span;
use thrushc_token_type::TokenType;
use thrushc_token_type::traits::TokenTypeExtensions;
use thrushc_typesystem::Type;

use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;

pub fn float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: FloatValue<'ctx>,
    rhs: FloatValue<'ctx>,
    operator: &TokenType,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let (lhs, rhs) = cast::float_together(context, lhs, rhs, span);

    match operator {
        TokenType::Plus => llvm_builder
            .build_float_add(lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile '+' operation!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
        TokenType::Minus => llvm_builder
            .build_float_sub(lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile '-' operation!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
        TokenType::Star => llvm_builder
            .build_float_mul(lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile '*' operation!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
        TokenType::Slash => llvm_builder
            .build_float_div(lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile '/' operation!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),

        TokenType::Arith => llvm_builder
            .build_float_rem(lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile '%' operation!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),

        op if op.is_logical_operator() => llvm_builder
            .build_float_compare(predicates::float(context, operator, span), lhs, rhs, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile comparison!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),

        _ => abort::abort_codegen(
            context,
            "Failed to compile without a valid operator!",
            span,
            std::path::PathBuf::from(file!()),
            line!(),
        ),
    }
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::Arith
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = codegen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = codegen::compile(context, binary.2, cast);

        return float_operation(
            context,
            lhs.into_float_value(),
            rhs.into_float_value(),
            operator,
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile float binary operation!",
        span,
        std::path::PathBuf::from(file!()),
        line!(),
    );
}

#[inline]
pub fn const_float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: FloatValue<'ctx>,
    rhs: FloatValue<'ctx>,
    operator: &TokenType,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let (lhs, rhs) = cast::const_float_together(lhs, rhs);

    match operator {
        TokenType::Plus => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number + rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Minus => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number - rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Star => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number * rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Slash => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number / rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Arith => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number % rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        op if op.is_logical_operator() => lhs
            .const_compare(predicates::float(context, operator, span), rhs)
            .into(),

        _ => {
            abort::abort_codegen(
                context,
                "Failed to compile constant float binary operation!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            );
        }
    }
}

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
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
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = codegen::compile_constant(context, binary.0, cast);
        let rhs: BasicValueEnum = codegen::compile_constant(context, binary.2, cast);

        return const_float_operation(
            context,
            lhs.into_float_value(),
            rhs.into_float_value(),
            operator,
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant float binary operation!",
        span,
        std::path::PathBuf::from(file!()),
        line!(),
    );
}
