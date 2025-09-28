use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_continue<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let continue_tk: &Token = ctx.consume(
        TokenType::Continue,
        String::from("Syntax error"),
        String::from("Expected 'continue' keyword."),
    )?;

    let span: Span = continue_tk.span;
    let scope: usize = ctx.get_scope();

    ctx.get_mut_control_ctx().set_unreacheable_code_scope(scope);

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Continue { span })
}

pub fn build_break<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let break_tk: &Token = ctx.consume(
        TokenType::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    let span: Span = break_tk.get_span();
    let scope: usize = ctx.get_scope();

    ctx.get_mut_control_ctx().set_unreacheable_code_scope(scope);

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Break { span })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)?;
    checks::check_inside_loop_state(ctx)
}
