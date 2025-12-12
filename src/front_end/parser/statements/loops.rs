use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::parser::statements::block;
use crate::front_end::parser::statements::local;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_for_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let for_tk: &Token = ctx.consume(
        TokenType::For,
        "Syntax error".into(),
        "Expected 'for' keyword.".into(),
    )?;

    let span: Span = for_tk.get_span();

    let local: Ast = local::build_local(ctx)?;

    let condition: Ast = expr::build_expression(ctx)?;
    let actions: Ast = expr::build_expression(ctx)?;

    let body: Ast = block::build_block(ctx)?;

    Ok(Ast::For {
        local: local.into(),
        condition: condition.into(),
        actions: actions.into(),
        block: body.into(),
        span,
    })
}

pub fn build_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let loop_tk: &Token = ctx.consume(
        TokenType::Loop,
        "Syntax error".into(),
        "Expected 'loop' keyword.".into(),
    )?;

    let span: Span = loop_tk.get_span();

    let block: Ast = block::build_block(ctx)?;

    Ok(Ast::Loop {
        block: block.into(),
        span,
    })
}

pub fn build_while_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let while_tk: &Token = ctx.consume(
        TokenType::While,
        "Syntax error".into(),
        "Expected 'while' keyword.".into(),
    )?;

    let span: Span = while_tk.get_span();

    let condition: Ast = expr::build_expr(ctx)?;
    let block: Ast = block::build_block(ctx)?;

    Ok(Ast::While {
        condition: condition.into(),
        block: block.into(),
        span,
    })
}
