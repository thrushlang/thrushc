use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
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
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    parser_context.consume(
        TokenType::Static,
        "Syntax error".into(),
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = parser_context.match_token(TokenType::Mut)?;

    let static_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = static_tk.get_lexeme();
    let ascii_name: &str = static_tk.get_ascii_lexeme();

    let span: Span = static_tk.get_span();

    parser_context.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let static_type: Type = typegen::build_type(parser_context)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_context, &[TokenType::Eq])?;

    parser_context.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(parser_context)?;

    let metadata: StaticMetadata = StaticMetadata::new(false, is_mutable);

    parser_context.get_mut_symbols().new_static(
        name,
        (static_type.clone(), metadata, attributes.clone()),
        span,
    )?;

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

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
