use thrustc_ast::{Ast, NodeId, traits::AstGetType};
use thrustc_errors::CompilationIssue;
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::{Type, traits::CastTypeExtensions};

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

        let expr: Ast = precedences::indirect::indirect_precedence(ctx)?;

        ctx.leave_expression();

        return Ok(Ast::UnaryOp {
            operator,
            node: expr.into(),
            kind: Type::Bool(span),
            is_pre: false,
            span,
            id: NodeId::new(),
        });
    }

    if ctx.match_token(TokenType::Minus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expr: Ast = precedences::indirect::indirect_precedence(ctx)?;
        let expr_type: &Type = expr.get_value_type()?;

        let kind: Type = expr_type.narrowing();

        ctx.leave_expression();

        return Ok(Ast::UnaryOp {
            operator,
            node: expr.clone().into(),
            kind,
            is_pre: false,
            span,
            id: NodeId::new(),
        });
    }

    if ctx.match_token(TokenType::Not)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expr: Ast = precedences::indirect::indirect_precedence(ctx)?;
        let expr_type: &Type = expr.get_value_type()?;

        ctx.leave_expression();

        return Ok(Ast::UnaryOp {
            operator,
            node: expr.clone().into(),
            kind: expr_type.clone(),
            is_pre: false,
            span,
            id: NodeId::new(),
        });
    }

    if ctx.match_token(TokenType::PlusPlus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expr: Ast = expressions::build_expr(ctx)?;
        let expr_type: &Type = expr.get_value_type()?;

        let unaryop: Ast = Ast::UnaryOp {
            operator,
            node: expr.clone().into(),
            kind: expr_type.clone(),
            is_pre: true,
            span,
            id: NodeId::new(),
        };

        ctx.leave_expression();

        return Ok(unaryop);
    }

    if ctx.match_token(TokenType::MinusMinus)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expr: Ast = expressions::build_expr(ctx)?;
        let expr_type: &Type = expr.get_value_type()?;

        let unaryop: Ast = Ast::UnaryOp {
            operator,
            node: expr.clone().into(),
            kind: expr_type.clone(),
            is_pre: true,
            span,
            id: NodeId::new(),
        };

        ctx.leave_expression();

        return Ok(unaryop);
    }

    let instr: Ast = precedences::indirect::indirect_precedence(ctx)?;

    ctx.leave_expression();

    Ok(instr)
}
