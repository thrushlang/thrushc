use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::property::PropertyMetadata;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::typesystem::{self, types::Type};

pub fn build_property<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let source_type: &Type = source.get_value_type()?;
    let metadata: PropertyMetadata = PropertyMetadata::new(source.is_allocated());

    let mut property_names: Vec<&str> = Vec::with_capacity(10);

    let first_property: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected property name.".into(),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.get_lexeme());

    while ctx.match_token(TokenType::Dot)? {
        let property: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected property name.".into(),
        )?;

        span = property.span;

        property_names.push(property.get_lexeme());
    }

    property_names.reverse();

    let decomposed_property: (Type, Vec<(Type, u32)>) =
        typesystem::property::decompose(ctx, 0, &source, property_names, source_type, span)?;

    let property_type: Type = decomposed_property.0;
    let indexes: Vec<(Type, u32)> = decomposed_property.1;

    Ok(Ast::Property {
        source: source.into(),
        indexes,
        kind: property_type,
        metadata,
        span,
    })
}
