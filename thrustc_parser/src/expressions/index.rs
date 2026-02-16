use thrustc_ast::{
    Ast,
    metadata::IndexMetadata,
    traits::{AstGetType, AstMutabilityExtensions},
};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token_type::TokenType;
use thrustc_typesystem::{Type, traits::IndexExtensions};

use crate::{ParserContext, expressions};

pub fn build_index<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
    span: Span,
) -> Result<Ast<'parser>, CompilationIssue> {
    let index_type: &Type = source.get_value_type()?;
    let is_mutable: bool = source.is_mutable();

    let index: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::RBracket,
        CompilationIssueCode::E0001,
        "Expected ']'.".into(),
    )?;

    let index_type: Type = Type::Ptr(
        Some(index_type.calculate_index_type(1).clone().into()),
        span,
    );

    Ok(Ast::Index {
        source: source.into(),
        index: index.into(),
        kind: index_type,
        metadata: IndexMetadata::new(is_mutable),
        span,
    })
}
