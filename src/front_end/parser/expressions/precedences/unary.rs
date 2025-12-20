use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::expressions::precedences::lower;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::{traits::CastTypeExtensions, types::Type};

pub fn unary_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    if ctx.match_token(TokenType::Bang)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = lower::lower_precedence(ctx)?;

        return Ok(Ast::UnaryOp {
            operator,
            expression: expression.into(),
            kind: Type::Bool(span),
            is_pre: false,
            span,
        });
    }

    if ctx.match_token(TokenType::Minus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = lower::lower_precedence(ctx)?;

        let expression_type: &Type = expression.get_value_type()?;
        let kind: Type = expression_type.narrowing();

        return Ok(Ast::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind,
            is_pre: false,
            span,
        });
    }

    if ctx.match_token(TokenType::Not)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = lower::lower_precedence(ctx)?;
        let expression_type: &Type = expression.get_value_type()?;

        return Ok(Ast::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: false,
            span,
        });
    }

    if ctx.match_token(TokenType::PlusPlus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = expressions::build_expr(ctx)?;

        let expression_type: &Type = expression.get_value_type()?;

        let unaryop: Ast = Ast::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: true,
            span,
        };

        return Ok(unaryop);
    }

    if ctx.match_token(TokenType::MinusMinus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = expressions::build_expr(ctx)?;
        let expression_type: &Type = expression.get_value_type()?;

        let unaryop: Ast = Ast::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: true,
            span,
        };

        return Ok(unaryop);
    }

    let instr: Ast = lower::lower_precedence(ctx)?;

    Ok(instr)
}
