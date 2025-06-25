use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, expression,
            stmts::{block, local},
        },
        types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_for_loop<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let for_tk: &Token = parser_ctx.consume(
        TokenType::For,
        String::from("Syntax error"),
        String::from("Expected 'for' keyword."),
    )?;

    let span: Span = for_tk.span;

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
            String::from("For loop must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    let local: Ast = local::build_local(parser_ctx)?;

    let cond: Ast = expression::build_expression(parser_ctx)?;
    let actions: Ast = expression::build_expression(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let body: Ast = block::build_block(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(Ast::For {
        local: local.into(),
        cond: cond.into(),
        actions: actions.into(),
        block: body.into(),
        span,
    })
}

pub fn build_loop<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let loop_tk: &Token = parser_ctx.consume(
        TokenType::Loop,
        String::from("Syntax error"),
        String::from("Expected 'loop' keyword."),
    )?;

    let loop_span: Span = loop_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            loop_span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Loop must be placed inside a function or a bind."),
            None,
            loop_span,
        ));
    }

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let block: Ast = block::build_block(parser_ctx)?;

    let scope: usize = parser_ctx.get_scope();

    if !block.has_break() && !block.has_return() && !block.has_continue() {
        parser_ctx
            .get_mut_control_ctx()
            .set_unreacheable_code_scope(scope);
    }

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(Ast::Loop {
        block: block.into(),
        span: loop_span,
    })
}

pub fn build_while_loop<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let while_tk: &Token = parser_ctx.consume(
        TokenType::While,
        String::from("Syntax error"),
        String::from("Expected 'while' keyword."),
    )?;

    let span: Span = while_tk.get_span();

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
            String::from("While loop must be placed inside a function or a structure method."),
            None,
            span,
        ));
    }

    let cond: Ast = expression::build_expr(parser_ctx)?;
    let block: Ast = block::build_block(parser_ctx)?;

    Ok(Ast::While {
        cond: cond.into(),
        block: block.into(),
        span,
    })
}
