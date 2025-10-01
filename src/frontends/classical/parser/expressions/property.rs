use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::{Ast, metadata::property::PropertyMetadata, types::AstEitherExpression},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::{self, types::Type},
    },
};

pub fn build_property<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: AstEitherExpression<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let source_expr_extract: (&Type, &Ast) = match source {
        (Some(ref any_reference), None, ..) => {
            let reference: &Ast = &any_reference.1;
            (reference.get_value_type()?, reference)
        }
        (None, Some(ref expr), ..) => (expr.get_value_type()?, expr),
        _ => {
            return Err(ThrushCompilerIssue::FrontEndBug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }
    };

    let source_type: &Type = source_expr_extract.0;
    let source_expr: &Ast = source_expr_extract.1;

    let metadata: PropertyMetadata = PropertyMetadata::new(source_expr.is_allocated());

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

    let decomposed: (Type, Vec<(Type, u32)>) =
        typesystem::property::decompose(0, property_names, source_type, ctx.get_symbols(), span)?;

    let property_type: Type = decomposed.0;
    let indexes: Vec<(Type, u32)> = decomposed.1;

    Ok(Ast::Property {
        source,
        indexes,
        kind: property_type,
        metadata,
        span,
    })
}
