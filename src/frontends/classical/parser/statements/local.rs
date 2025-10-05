use inkwell::AtomicOrdering;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, builder, checks, expr, typegen},
        types::{
            ast::{Ast, metadata::local::LocalMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_local<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let local_tk: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected identifier."),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();
    let span: Span = local_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let local_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile, atom_ord);

        ctx.get_mut_symbols()
            .new_local(name, (local_type.clone(), metadata, span), span)?;

        return Ok(Ast::Local {
            name,
            ascii_name,
            kind: local_type,
            value: None,
            attributes,
            metadata,
            span,
        });
    }

    let metadata: LocalMetadata = LocalMetadata::new(false, is_mutable, is_volatile, atom_ord);

    ctx.get_mut_symbols()
        .new_local(name, (local_type.clone(), metadata, span), span)?;

    ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expr::build_expression(ctx)?;

    let local: Ast = Ast::Local {
        name,
        ascii_name,
        kind: local_type,
        value: Some(value.into()),
        attributes,
        metadata,
        span,
    };

    Ok(local)
}

fn check_state(ctx: &mut ParserContext<'_>) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
