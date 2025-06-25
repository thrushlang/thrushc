use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::reference},
        types::{
            ast::Ast,
            lexer::{ThrushType, decompose_struct_property},
            parser::stmts::traits::TokenExtensions,
        },
    },
};

pub fn build_property<'parser>(
    parser_context: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let reference: Ast = reference::build_reference(parser_context, name, span)?;
    let reference_type: ThrushType = reference.get_any_type()?.clone();

    let mut property_names: Vec<&str> = Vec::with_capacity(10);

    let first_property: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected property name."),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.get_lexeme());

    while parser_context.match_token(TokenType::Dot)? {
        let property: &Token = parser_context.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected property name."),
        )?;

        span = property.span;

        property_names.push(property.get_lexeme());
    }

    property_names.reverse();

    let decomposed: (ThrushType, Vec<(ThrushType, u32)>) = decompose_struct_property(
        0,
        property_names,
        reference_type,
        parser_context.get_symbols(),
        span,
    )?;

    Ok(Ast::Property {
        name,
        reference: reference.into(),
        indexes: decomposed.1,
        kind: decomposed.0,
        span,
    })
}
