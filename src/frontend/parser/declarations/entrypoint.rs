use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, stmts::block},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_main<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let span: Span = parser_context.previous().span;

    if parser_context.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            "Duplicated entrypoint".into(),
            "The language not support two entrypoints. :>".into(),
            None,
            parser_context.previous().get_span(),
        ));
    }

    parser_context.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    parser_context.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    parser_context.consume(
        TokenType::U32,
        "Syntax error".into(),
        "Expected 'u32'.".into(),
    )?;

    parser_context.get_mut_control_ctx().set_entrypoint(true);

    parser_context
        .get_mut_type_ctx()
        .set_function_type(Type::U32);

    Ok(Ast::EntryPoint {
        body: block::build_block(parser_context)?.into(),
        span,
    })
}
