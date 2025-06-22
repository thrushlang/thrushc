use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, expression,
            expressions::precedences::equality::{self},
        },
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn unary_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    if parser_context.match_token(TokenType::Bang)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let expression: ThrushStatement = equality::equality_precedence(parser_context)?;

        return Ok(ThrushStatement::UnaryOp {
            operator,
            expression: expression.into(),
            kind: ThrushType::Bool,
            is_pre: false,
            span,
        });
    }

    if parser_context.match_token(TokenType::Minus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let mut expression: ThrushStatement = equality::equality_precedence(parser_context)?;

        expression.cast_signess(operator);

        let expression_type: &ThrushType = expression.get_value_type()?;

        return Ok(ThrushStatement::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: false,
            span,
        });
    }

    if parser_context.match_token(TokenType::PlusPlus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let expression: ThrushStatement = expression::build_expr(parser_context)?;

        let expression_type: &ThrushType = expression.get_value_type()?;

        let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
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

        let expression: ThrushStatement = expression::build_expr(parser_context)?;
        let expression_type: &ThrushType = expression.get_value_type()?;

        let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: true,
            span,
        };

        return Ok(unaryop);
    }

    let instr: ThrushStatement = equality::equality_precedence(parser_context)?;

    Ok(instr)
}
