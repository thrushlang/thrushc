use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_continue<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    let continue_tk: &Token = parser_context.consume(
        TokenType::Continue,
        String::from("Syntax error"),
        String::from("Expected 'continue' keyword."),
    )?;

    let span: Span = continue_tk.span;
    let scope: usize = parser_context.get_scope();

    parser_context
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Continue { span })
}

pub fn build_break<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    let break_tk: &Token = parser_context.consume(
        TokenType::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    let span: Span = break_tk.get_span();
    let scope: usize = parser_context.get_scope();

    parser_context
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Break { span })
}

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unreachable for execution.".into(),
            None,
            parser_context.peek().get_span(),
        ));
    }

    if !parser_context.get_control_ctx().get_inside_function() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "A loop flow control must be inside a function.".into(),
            None,
            parser_context.peek().get_span(),
        ));
    }

    if !parser_context.get_control_ctx().is_inside_loop() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "A loop flow control must be inside one.".into(),
            None,
            parser_context.peek().get_span(),
        ));
    }

    Ok(())
}
