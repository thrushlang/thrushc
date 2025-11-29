use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::index::IndexMetadata;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstMutabilityExtensions;
use crate::front_end::typesystem::traits::IndexExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_index<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
    span: Span,
) -> Result<Ast<'parser>, CompilationIssue> {
    let index_type: &Type = source.get_value_type()?;
    let is_mutable: bool = source.is_mutable();

    let index: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::RBracket,
        "Syntax error".into(),
        "Expected ']'.".into(),
    )?;

    let index_type: Type = Type::Ptr(Some(index_type.calculate_index_type(1).clone().into()));

    Ok(Ast::Index {
        source: source.into(),
        index: index.into(),
        kind: index_type,
        metadata: IndexMetadata::new(is_mutable),
        span,
    })
}
