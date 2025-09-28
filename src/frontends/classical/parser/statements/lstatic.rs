use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, checks, expr, typegen},
        types::{
            ast::{Ast, metadata::staticvar::StaticMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_static<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Static,
        "Syntax error".into(),
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

    let is_lazy: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatible: bool = ctx.match_token(TokenType::Volatile)?;

    let static_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = static_tk.get_lexeme();
    let ascii_name: &str = static_tk.get_ascii_lexeme();

    let span: Span = static_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let static_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(ctx)?;

    let metadata: StaticMetadata = StaticMetadata::new(false, is_mutable, is_lazy, is_volatible);

    if let Err(error) = ctx.get_mut_symbols().new_static(
        name,
        (static_type.clone(), metadata, attributes.clone()),
        span,
    ) {
        ctx.add_silent_error(error);
    }

    Ok(Ast::Static {
        name,
        ascii_name,
        kind: static_type,
        value: value.into(),
        attributes,
        metadata,
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
