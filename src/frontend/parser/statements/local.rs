use inkwell::AtomicOrdering;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::attributes;
use crate::frontend::parser::builder;
use crate::frontend::parser::checks;
use crate::frontend::parser::expr;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::local::LocalMetadata;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::types::Type;

pub fn build_local<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Local,
        "Syntax error".into(),
        "Expected 'local' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let local_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected identifier.".into(),
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
