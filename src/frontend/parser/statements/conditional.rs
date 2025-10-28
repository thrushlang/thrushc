use std::rc::Rc;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::checks;
use crate::frontend::parser::expr;
use crate::frontend::parser::statements::block;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;

pub fn build_conditional<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let if_tk: &Token = ctx.consume(
        TokenType::If,
        "Syntax error".into(),
        "Expected 'if' keyword.".into(),
    )?;

    let span: Span = if_tk.get_span();

    let condition: Ast = expr::build_expr(ctx)?;
    let block: Ast = block::build_block(ctx)?;

    let mut elseif: Vec<Ast> = Vec::with_capacity(10);

    while ctx.match_token(TokenType::Elif)? {
        let span: Span = ctx.previous().span;

        let condition: Ast = expr::build_expr(ctx)?;
        let block: Ast = block::build_block(ctx)?;

        if block.is_empty_block() {
            continue;
        }

        elseif.push(Ast::Elif {
            condition: condition.into(),
            block: block.into(),
            span,
        });
    }

    let mut anyway: Option<Rc<Ast>> = None;

    if ctx.match_token(TokenType::Else)? {
        let span: Span = ctx.previous().span;
        let block: Ast = block::build_block(ctx)?;

        if !block.is_empty_block() {
            anyway = Some(
                Ast::Else {
                    block: block.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(Ast::If {
        condition: condition.into(),
        block: block.into(),
        elseif,
        anyway,
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
