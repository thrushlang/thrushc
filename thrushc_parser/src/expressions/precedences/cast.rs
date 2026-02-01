use thrushc_ast::{
    Ast,
    metadata::CastingMetadata,
    traits::{AstConstantExtensions, AstGetType, AstMemoryExtensions},
};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::traits::TokenExtensions;
use thrushc_token_type::TokenType;
use thrushc_typesystem::{Type, traits::TypeIsExtensions};

use crate::{ParserContext, expressions::precedences, typegen};

pub fn cast_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expression: Ast = precedences::index::index_precedence(ctx)?;

    if ctx.match_token(TokenType::As)? {
        let span: Span = ctx.previous().get_span();
        let expression_type: &Type = expression.get_value_type()?;

        let cast: Type = typegen::build_type(ctx, false)?;

        let is_constant: bool = expression.is_constant_value();
        let is_allocated: bool = expression.is_allocated() || expression_type.is_ptr_type();

        expression = Ast::As {
            from: expression.into(),
            cast,
            metadata: CastingMetadata::new(is_constant, is_allocated),
            span,
        };
    }

    ctx.leave_expression();

    Ok(expression)
}
