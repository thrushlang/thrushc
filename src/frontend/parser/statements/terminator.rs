use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_return<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let return_tk: &Token = ctx.consume(
        TokenType::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    let span: Span = return_tk.get_span();

    if ctx.match_token(TokenType::SemiColon)? {
        return Ok(Ast::Return {
            expression: None,
            kind: Type::Void,
            span,
        });
    }

    let value: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Return {
        expression: Some(value.into()),
        kind: ctx.get_type_ctx().get_function_type().clone(),
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
