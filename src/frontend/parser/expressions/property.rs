use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::{Ast, types::AstEitherExpression},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::{self, types::Type},
    },
};

pub fn build_property<'parser>(
    parser_context: &mut ParserContext<'parser>,
    source: AstEitherExpression<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let source_type: &Type = match source {
        (Some(ref any_reference), None) => {
            let reference: &Ast = &any_reference.1;
            reference.get_value_type()?
        }
        (None, Some(ref expr)) => expr.get_value_type()?,
        _ => {
            return Err(ThrushCompilerIssue::Bug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }
    };

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

    let decomposed: (Type, Vec<(Type, u32)>) = typesystem::types::decompose_property(
        0,
        property_names,
        source_type,
        parser_context.get_symbols(),
        span,
    )?;

    let property_type: Type = decomposed.0;
    let indexes: Vec<(Type, u32)> = decomposed.1;

    Ok(Ast::Property {
        source,
        indexes,
        kind: property_type,
        span,
    })
}
