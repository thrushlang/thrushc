use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, checks, statements::block},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn build_entrypoint<'parser>(
    parser_context: &mut ParserContext<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_double_entrypoint_state(parser_context)?;

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

    parser_context.get_mut_control_ctx().set_has_entrypoint();

    parser_context
        .get_mut_type_ctx()
        .set_function_type(Type::U32);

    Ok(Ast::EntryPoint {
        body: block::build_block(parser_context)?.into(),
        span,
    })
}
