use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::index::IndexMetadata;
use crate::front_end::typesystem::traits::TypeExtensions;
use crate::front_end::typesystem::types::Type;

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
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBracket,
        "Syntax error".into(),
        "Expected ']'.".into(),
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
