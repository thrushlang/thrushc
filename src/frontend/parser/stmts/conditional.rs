use std::rc::Rc;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression, stmts::block},
        types::parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
    },
};

pub fn build_conditional<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let if_tk: &Token = parser_ctx.consume(
        TokenType::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    let span: Span = if_tk.get_span();

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Conditionals must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let if_condition: ThrushStatement = expression::build_expr(parser_ctx)?;
    let if_body: ThrushStatement = block::build_block(parser_ctx)?;

    let mut elfs: Vec<ThrushStatement> = Vec::with_capacity(10);

    while parser_ctx.match_token(TokenType::Elif)? {
        let span: Span = parser_ctx.previous().span;

        let elif_condition: ThrushStatement = expression::build_expr(parser_ctx)?;

        let elif_body: ThrushStatement = block::build_block(parser_ctx)?;

        if !elif_body.has_block() {
            continue;
        }

        elfs.push(ThrushStatement::Elif {
            cond: elif_condition.into(),
            block: elif_body.into(),
            span,
        });
    }

    let mut otherwise: Option<Rc<ThrushStatement>> = None;

    if parser_ctx.match_token(TokenType::Else)? {
        let span: Span = parser_ctx.previous().span;
        let else_body: ThrushStatement = block::build_block(parser_ctx)?;

        if else_body.has_block() {
            otherwise = Some(
                ThrushStatement::Else {
                    block: else_body.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(ThrushStatement::If {
        cond: if_condition.into(),
        block: if_body.into(),
        elfs,
        otherwise,
        span,
    })
}
