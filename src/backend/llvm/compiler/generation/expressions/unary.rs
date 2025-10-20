use std::path::PathBuf;

use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::generation::cast;
use crate::backend::llvm::compiler::memory::SymbolAllocated;
use crate::backend::llvm::compiler::{abort, codegen, constgen, typegen};

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::repr::UnaryOperation;
use crate::frontend::typesystem::types::Type;

use inkwell::types::FloatType;
use inkwell::values::PointerValue;
use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match unary {
        (
            TokenType::PlusPlus | TokenType::MinusMinus,
            _,
            Ast::Reference {
                name, kind, span, ..
            },
        ) => self::compile_increment_decrement_ref(context, name, unary.0, kind, *span, cast_type),
        (TokenType::PlusPlus | TokenType::MinusMinus, _, expr) => {
            self::compile_increment_decrement(context, unary.0, expr, cast_type)
        }

        (TokenType::Bang, _, expr) => self::compile_logical_negation(context, expr, cast_type),
        (TokenType::Minus, _, expr) => self::compile_arithmetic_negation(context, expr, cast_type),
        (TokenType::Not, _, expr) => self::compile_bitwise_not(context, expr, cast_type),

        what => abort::abort_codegen(
            context,
            "Unknown unary operation!",
            what.2.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    match unary {
        (TokenType::PlusPlus | TokenType::MinusMinus, _, expr) => {
            self::compile_increment_decrement_const(context, unary.0, expr, cast_type)
        }
        (TokenType::Bang, _, expr) => {
            self::compile_logical_negation_const(context, expr, cast_type)
        }
        (TokenType::Minus, _, expr) => {
            self::compile_arithmetic_negation_const(context, expr, cast_type)
        }
        (TokenType::Not, _, expr) => {
            self::compile_arithmetic_bitwise_not_const(context, expr, cast_type)
        }

        what => abort::abort_codegen(
            context,
            "Unknown unary operation!",
            what.2.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

fn compile_increment_decrement_ref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    operator: &TokenType,
    kind: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let symbol: SymbolAllocated = context.get_table().get_symbol(name);

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_int_nsw_add(int, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '++' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    })
                    .into(),

                TokenType::MinusMinus => llvm_builder
                    .build_int_nsw_sub(int, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '--' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
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

            let result: BasicValueEnum =
                cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);

            symbol.store(context, result);

            result
        }
        _ => {
            let float: FloatValue = symbol.load(context).into_float_value();

            let modifier: FloatValue = typegen::generate(llvm_context, kind)
                .into_float_type()
                .const_float(1.0);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '++' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    })
                    .into(),
                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '--' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
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

            let result: BasicValueEnum =
                cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);

            symbol.store(context, result);

            result
        }
    }
}

fn compile_increment_decrement<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    operator: &TokenType,
    expression: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = codegen::compile(context, expression, cast_type);
    let kind: &Type = expression.llvm_get_type(context);

    let span: Span = expression.get_span();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_int_nsw_add(int, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '++' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    })
                    .into(),

                TokenType::MinusMinus => llvm_builder
                    .build_int_nsw_sub(int, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '--' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
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

            cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result)
        }
        _ => {
            let float: FloatValue = value.into_float_value();

            let modifier: FloatValue = float.get_type().const_float(1.0);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '++' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    })
                    .into(),

                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile '--' operation!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        )
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

            cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result)
        }
    }
}

fn compile_logical_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = codegen::compile(context, expr, cast_type);
    let kind: &Type = expr.llvm_get_type(context);

    let span: Span = expr.get_span();

    match kind {
        kind if kind.is_bool_type() => {
            let int: IntValue = value.into_int_value();

            if let Ok(result) = llvm_builder.build_not(int, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);
            }

            int.into()
        }

        _ => abort::abort_codegen(
            context,
            "Unknown type for logical negation!",
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

fn compile_arithmetic_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = codegen::compile(context, expr, cast_type);
    let kind: &Type = expr.llvm_get_type(context);

    let span: Span = expr.get_span();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            if let Ok(result) = llvm_builder.build_int_neg(int, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);
            }

            int.into()
        }

        _ => {
            let float: FloatValue = value.into_float_value();

            if let Ok(result) = llvm_builder.build_float_neg(float, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);
            }

            float.into()
        }
    }
}

fn compile_bitwise_not<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = codegen::compile(context, expr, cast_type);
    let kind: &Type = expr.llvm_get_type(context);

    let span: Span = expr.get_span();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            if let Ok(result) = llvm_builder.build_not(int, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);
            }

            int.into()
        }

        _ => {
            let ptr: PointerValue = value.into_pointer_value();

            if let Ok(result) = llvm_builder.build_not(ptr, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast_type, kind, result, span).unwrap_or(result);
            }

            ptr.into()
        }
    }
}

fn compile_increment_decrement_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    operator: &TokenType,
    expression: &'ctx Ast,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expression, cast_type);

    let kind: &Type = expression.llvm_get_type(context);
    let span: Span = expression.get_span();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            match operator {
                TokenType::PlusPlus => int.const_add(modifier).into(),
                TokenType::MinusMinus => int.const_sub(modifier).into(),

                _ => abort::abort_codegen(
                    context,
                    "Expected '++' or '--' operation!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                ),
            }
        }
        _ => {
            let float: FloatValue = value.into_float_value();

            match operator {
                TokenType::PlusPlus => {
                    if let Some(constant_float) = float.get_constant() {
                        let value: f64 = constant_float.0;
                        let new_value: f64 = value + 1.0;

                        return float.get_type().const_float(new_value).into();
                    }

                    float.into()
                }

                TokenType::MinusMinus => {
                    if let Some(constant_float) = float.get_constant() {
                        let value: f64 = constant_float.0;
                        let new_value: f64 = value - 1.0;

                        return float.get_type().const_float(new_value).into();
                    }

                    float.into()
                }

                _ => abort::abort_codegen(
                    context,
                    "Failed to compile without a valid operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                ),
            }
        }
    }
}

fn compile_logical_negation_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expr, cast_type);

    let kind: &Type = expr.llvm_get_type(context);
    let span: Span = expr.get_span();

    match kind {
        kind if kind.is_bool_type() => {
            let int: IntValue = value.into_int_value();
            int.const_not().into()
        }

        _ => abort::abort_codegen(
            context,
            "Failed to compile without a valid operator!",
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}

fn compile_arithmetic_negation_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expr, cast_type);
    let kind: &Type = expr.llvm_get_type(context);

    match kind {
        kind if kind.is_integer_type() => value.into_int_value().const_neg().into(),
        _ => {
            let mut float: FloatValue = value.into_float_value();
            let float_type: FloatType = float.get_type();

            if let Some(float_value) = float.get_constant() {
                float = float_type.const_float(-float_value.0);
            }

            float.into()
        }
    }
}

fn compile_arithmetic_bitwise_not_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = constgen::compile(context, expr, cast_type);
    let kind: &Type = expr.llvm_get_type(context);

    match kind {
        kind if kind.is_integer_type() => value.into_int_value().const_not().into(),

        _ => abort::abort_codegen(
            context,
            "Failed to compile without a valid operator!",
            expr.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
