use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, checks, statements::block},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn build_entrypoint<'parser>(
    ctx: &mut ParserContext<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_double_entrypoint_state(ctx)?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    ctx.consume(
        TokenType::U32,
        "Syntax error".into(),
        "Expected 'u32'.".into(),
    )?;

    ctx.get_mut_control_ctx().set_has_entrypoint();
    ctx.get_mut_type_ctx().set_function_type(Type::U32);

    Ok(Ast::EntryPoint {
        body: block::build_block(ctx)?.into(),
        span,
    })
}
