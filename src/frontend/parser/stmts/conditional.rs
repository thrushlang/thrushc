use std::rc::Rc;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr, stmts::block},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_conditional<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    let if_tk: &Token = parser_context.consume(
        TokenType::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    let span: Span = if_tk.get_span();

    let if_condition: Ast = expr::build_expr(parser_context)?;
    let if_body: Ast = block::build_block(parser_context)?;

    let mut elfs: Vec<Ast> = Vec::with_capacity(10);

    while parser_context.match_token(TokenType::Elif)? {
        let span: Span = parser_context.previous().span;

        let elif_condition: Ast = expr::build_expr(parser_context)?;

        let elif_body: Ast = block::build_block(parser_context)?;

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

    if parser_context.match_token(TokenType::Else)? {
        let span: Span = parser_context.previous().span;
        let else_body: Ast = block::build_block(parser_context)?;

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

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
