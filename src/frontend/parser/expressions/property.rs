use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::{Ast, metadata::property::PropertyMetadata},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::{self, types::Type},
    },
};

pub fn build_property<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let source_type: &Type = source.get_value_type()?;
    let metadata: PropertyMetadata = PropertyMetadata::new(source.is_allocated());

    let mut property_names: Vec<&str> = Vec::with_capacity(10);

    let first_property: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected property name."),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.get_lexeme());

    while ctx.match_token(TokenType::Dot)? {
        let property: &Token = ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected property name."),
        )?;

        span = property.span;

        property_names.push(property.get_lexeme());
    }

    property_names.reverse();

    let decomposed: (Type, Vec<(Type, u32)>) = typesystem::property::decompose(
        0,
        &source,
        property_names,
        source_type,
        ctx.get_symbols(),
        span,
    )?;

    let property_type: Type = decomposed.0;
    let indexes: Vec<(Type, u32)> = decomposed.1;

    Ok(Ast::Property {
        source: source.into(),
        indexes,
        kind: property_type,
        metadata,
        span,
    })
}
