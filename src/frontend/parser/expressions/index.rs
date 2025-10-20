use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::ast::{Ast, metadata::index::IndexMetadata},
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn build_index<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let index_type: &Type = source.get_value_type()?;
    let is_mutable: bool = source.is_mutable();

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

    let index_type: Type = Type::Ptr(Some(
        index_type.get_type_with_depth(indexes.len()).clone().into(),
    ));

    Ok(Ast::Index {
        source: source.into(),
        indexes,
        kind: index_type,
        metadata: IndexMetadata::new(is_mutable),
        span,
    })
}
