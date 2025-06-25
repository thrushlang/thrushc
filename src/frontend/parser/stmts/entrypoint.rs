use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, stmts::block},
        types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_main<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let span: Span = parser_ctx.previous().span;

    if parser_ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Duplicated entrypoint"),
            String::from("The language not support two entrypoints. :>"),
            None,
            parser_ctx.previous().get_span(),
        ));
    }

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    parser_ctx.get_mut_control_ctx().set_entrypoint(true);

    Ok(Ast::EntryPoint {
        body: block::build_block(parser_ctx)?.into(),
        span,
    })
}
