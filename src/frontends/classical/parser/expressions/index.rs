use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::ast::{Ast, metadata::index::IndexMetadata, types::AstEitherExpression},
        typesystem::{traits::IndexTypeExtensions, types::Type},
    },
};

pub fn build_index<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: AstEitherExpression<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let index_type: &Type = match source {
        (Some(ref any_reference), None) => {
            let reference: &Ast = &any_reference.1;
            reference.get_value_type()?
        }
        (None, Some(ref expr)) => expr.get_value_type()?,
        _ => {
            return Err(ThrushCompilerIssue::FrontEndBug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }
    };

    let is_mutable: bool = match source {
        (Some(ref any_reference), None) => {
            let reference: &Ast = &any_reference.1;
            reference.is_mutable()
        }
        (None, Some(ref expr)) => expr.is_mutable(),
        _ => {
            return Err(ThrushCompilerIssue::FrontEndBug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }
    };

    let mut indexes: Vec<Ast> = Vec::with_capacity(50);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let indexe: Ast = expr::build_expr(ctx)?;
        indexes.push(indexe);

        if ctx.check(TokenType::RBracket) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    let index_type: Type = if index_type.is_ptr_type() {
        Type::Ptr(Some(
            index_type.get_aprox_type(indexes.len()).clone().into(),
        ))
    } else {
        Type::Mut(index_type.get_aprox_type(indexes.len()).clone().into())
    };

    Ok(Ast::Index {
        source,
        indexes,
        kind: index_type,
        metadata: IndexMetadata::new(is_mutable),
        span,
    })
}
