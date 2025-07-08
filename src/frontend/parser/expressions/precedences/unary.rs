use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, expr,
            expressions::precedences::equality::{self},
        },
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::{traits::CastTypeExtensions, types::Type},
    },
};

pub fn unary_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    if parser_context.match_token(TokenType::Bang)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let expression: Ast = equality::equality_precedence(parser_context)?;

        return Ok(Ast::UnaryOp {
            operator,
            expression: expression.into(),
            kind: Type::Bool,
            is_pre: false,
            span,
        });
    }

    if parser_context.match_token(TokenType::Minus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = equality::equality_precedence(parser_context)?;

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

    if parser_context.match_token(TokenType::PlusPlus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = expr::build_expr(parser_context)?;

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

    if parser_context.match_token(TokenType::MinusMinus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: Ast = expr::build_expr(parser_context)?;
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

    let instr: Ast = equality::equality_precedence(parser_context)?;

    Ok(instr)
}
