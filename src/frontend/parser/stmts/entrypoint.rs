use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, stmts::block},
        types::parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
    },
};

pub fn build_main<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    Ok(ThrushStatement::EntryPoint {
        body: block::build_block(parser_ctx)?.into(),
        span,
    })
}
