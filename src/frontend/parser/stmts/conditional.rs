use std::rc::Rc;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression, stmts::block},
        types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_conditional<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let if_tk: &Token = parser_ctx.consume(
        TokenType::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    let span: Span = if_tk.get_span();

    if !parser_ctx.get_control_ctx().get_inside_function() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Conditionals must be placed inside a function."),
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

    let if_condition: Ast = expression::build_expr(parser_ctx)?;
    let if_body: Ast = block::build_block(parser_ctx)?;

    let mut elfs: Vec<Ast> = Vec::with_capacity(10);

    while parser_ctx.match_token(TokenType::Elif)? {
        let span: Span = parser_ctx.previous().span;

        let elif_condition: Ast = expression::build_expr(parser_ctx)?;

        let elif_body: Ast = block::build_block(parser_ctx)?;

        if !elif_body.has_block() {
            continue;
        }

        elfs.push(Ast::Elif {
            cond: elif_condition.into(),
            block: elif_body.into(),
            span,
        });
    }

    let mut otherwise: Option<Rc<Ast>> = None;

    if parser_ctx.match_token(TokenType::Else)? {
        let span: Span = parser_ctx.previous().span;
        let else_body: Ast = block::build_block(parser_ctx)?;

        if else_body.has_block() {
            otherwise = Some(
                Ast::Else {
                    block: else_body.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(Ast::If {
        cond: if_condition.into(),
        block: if_body.into(),
        elfs,
        otherwise,
        span,
    })
}
