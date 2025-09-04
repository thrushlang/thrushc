use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, checks, expr, typegen},
        types::{
            ast::{Ast, metadata::local::LocalMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_local<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_ctx)?;

    parser_ctx.consume(
        TokenType::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;
    let is_volatile: bool = parser_ctx.match_token(TokenType::Volatile)?;

    let local_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected identifier."),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();

    let span: Span = local_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let local_type: Type = typegen::build_type(parser_ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if parser_ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile);

        parser_ctx
            .get_mut_symbols()
            .new_local(name, (local_type.clone(), metadata, span), span)?;

        return Ok(Ast::Local {
            name,
            ascii_name,
            kind: local_type,
            value: Ast::Null { span }.into(),
            attributes,
            metadata,
            span,
        });
    }

    let metadata: LocalMetadata = LocalMetadata::new(false, is_mutable, is_volatile);

    parser_ctx
        .get_mut_symbols()
        .new_local(name, (local_type.clone(), metadata, span), span)?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expr::build_expression(parser_ctx)?;

    let local: Ast = Ast::Local {
        name,
        ascii_name,
        kind: local_type,
        value: value.into(),
        attributes,
        metadata,
        span,
    };

    Ok(local)
}

fn check_state(parser_ctx: &mut ParserContext<'_>) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_ctx)?;
    checks::check_inside_function_state(parser_ctx)
}
