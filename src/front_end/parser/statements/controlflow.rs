use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::checks;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_continue<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let continue_tk: &Token = ctx.consume(
        TokenType::Continue,
        "Syntax error".into(),
        "Expected 'continue' keyword.".into(),
    )?;

    let span: Span = continue_tk.span;
    let scope: usize = ctx.get_scope();

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Continue { span })
}

pub fn build_break<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let break_tk: &Token = ctx.consume(
        TokenType::Break,
        "Syntax error".into(),
        "Expected 'break' keyword.".into(),
    )?;

    let span: Span = break_tk.get_span();
    let scope: usize = ctx.get_scope();

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Break { span })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_inside_function_state(ctx)?;
    checks::check_inside_loop_state(ctx)
}
