use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, stmts::block},
        types::{ast::Ast, lexer::Type, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_main<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let span: Span = parser_ctx.previous().span;

    if parser_ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            "Duplicated entrypoint".into(),
            "The language not support two entrypoints. :>".into(),
            None,
            parser_ctx.previous().get_span(),
        ));
    }

    parser_ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    parser_ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    parser_ctx.consume(
        TokenType::U32,
        "Syntax error".into(),
        "Expected 'u32'.".into(),
    )?;

    parser_ctx.get_mut_control_ctx().set_entrypoint(true);

    parser_ctx.get_mut_type_ctx().set_function_type(Type::U32);

    Ok(Ast::EntryPoint {
        body: block::build_block(parser_ctx)?.into(),
        span,
    })
}
