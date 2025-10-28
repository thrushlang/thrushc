use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::checks;
use crate::frontend::parser::expr;
use crate::frontend::parser::statements::block;
use crate::frontend::parser::statements::local;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;

pub fn build_for_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let for_tk: &Token = ctx.consume(
        TokenType::For,
        "Syntax error".into(),
        "Expected 'for' keyword.".into(),
    )?;

    let span: Span = for_tk.span;

    let local: Ast = local::build_local(ctx)?;

    let cond: Ast = expr::build_expression(ctx)?;
    let actions: Ast = expr::build_expression(ctx)?;

    ctx.get_mut_control_ctx().increment_loop_depth();

    let body: Ast = block::build_block(ctx)?;

    ctx.get_mut_control_ctx().decrement_loop_depth();

    Ok(Ast::For {
        local: local.into(),
        cond: cond.into(),
        actions: actions.into(),
        block: body.into(),
        span,
    })
}

pub fn build_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let loop_tk: &Token = ctx.consume(
        TokenType::Loop,
        "Syntax error".into(),
        "Expected 'loop' keyword.".into(),
    )?;

    let loop_span: Span = loop_tk.span;

    ctx.get_mut_control_ctx().increment_loop_depth();

    let block: Ast = block::build_block(ctx)?;

    let scope: usize = ctx.get_scope();

    if !block.has_break() && !block.has_return() && !block.has_continue() {
        ctx.get_mut_control_ctx().set_unreacheable_code_scope(scope);
    }

    ctx.get_mut_control_ctx().decrement_loop_depth();

    Ok(Ast::Loop {
        block: block.into(),
        span: loop_span,
    })
}

pub fn build_while_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let while_tk: &Token = ctx.consume(
        TokenType::While,
        "Syntax error".into(),
        "Expected 'while' keyword.".into(),
    )?;

    let span: Span = while_tk.get_span();

    let cond: Ast = expr::build_expr(ctx)?;

    ctx.get_mut_control_ctx().increment_loop_depth();

    let block: Ast = block::build_block(ctx)?;

    ctx.get_mut_control_ctx().decrement_loop_depth();

    Ok(Ast::While {
        cond: cond.into(),
        block: block.into(),
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
