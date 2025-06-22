use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
    },
};

pub fn build_continue<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let continue_tk: &Token = parser_ctx.consume(
        TokenType::Continue,
        String::from("Syntax error"),
        String::from("Expected 'continue' keyword."),
    )?;

    let span: Span = continue_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Continue must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(ThrushStatement::Continue { span })
}

pub fn build_break<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let break_tk: &Token = parser_ctx.consume(
        TokenType::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    let span: Span = break_tk.get_span();

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Break must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(ThrushStatement::Break { span })
}
