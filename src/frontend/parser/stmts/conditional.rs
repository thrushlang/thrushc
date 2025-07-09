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

    let mut elseif: Vec<Ast> = Vec::with_capacity(10);

    while parser_context.match_token(TokenType::Elif)? {
        let span: Span = parser_context.previous().span;

        let condition: Ast = expr::build_expr(parser_context)?;
        let block: Ast = block::build_block(parser_context)?;

        if !block.has_block() {
            continue;
        }

        elseif.push(Ast::Elif {
            condition: condition.into(),
            block: block.into(),
            span,
        });
    }

    let mut anyway: Option<Rc<Ast>> = None;

    if parser_context.match_token(TokenType::Else)? {
        let span: Span = parser_context.previous().span;
        let else_body: Ast = block::build_block(parser_context)?;

        if else_body.has_block() {
            anyway = Some(
                Ast::Else {
                    block: else_body.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(Ast::If {
        condition: if_condition.into(),
        block: if_body.into(),
        elseif,
        anyway,
        span,
    })
}

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
