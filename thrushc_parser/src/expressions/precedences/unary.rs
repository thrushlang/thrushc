use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::{Type, traits::CastTypeExtensions};

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

pub fn unary_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    if ctx.match_token(TokenType::Bang)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = precedences::lower::lower_precedence(ctx)?;

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

        let expression: Ast = precedences::lower::lower_precedence(ctx)?;

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

        let expression: Ast = precedences::lower::lower_precedence(ctx)?;
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

    let instr: Ast = precedences::lower::lower_precedence(ctx)?;

    Ok(instr)
}
