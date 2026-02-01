use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::{Type, traits::CastTypeExtensions};

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

pub fn unary_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    if ctx.match_token(TokenType::Bang)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = precedences::indirect::indirect_precedence(ctx)?;

        ctx.leave_expression();

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

        let expression: Ast = precedences::indirect::indirect_precedence(ctx)?;

        let expression_type: &Type = expression.get_value_type()?;
        let kind: Type = expression_type.narrowing();

        ctx.leave_expression();

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

        let expression: Ast = precedences::indirect::indirect_precedence(ctx)?;
        let expression_type: &Type = expression.get_value_type()?;

        ctx.leave_expression();

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

        ctx.leave_expression();

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

        ctx.leave_expression();

        return Ok(unaryop);
    }

    let instr: Ast = precedences::indirect::indirect_precedence(ctx)?;

    ctx.leave_expression();

    Ok(instr)
}
