use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::expressions::precedences::index;
use crate::front_end::parser::{ParserContext, typegen};
use crate::front_end::types::ast::traits::{
    AstConstantExtensions, AstGetType, AstMemoryExtensions,
};
use crate::front_end::types::ast::{Ast, metadata::cast::CastMetadata};
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::types::Type;

pub fn cast_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = index::index_precedence(ctx)?;

    if ctx.match_token(TokenType::As)? {
        let span: Span = ctx.previous().get_span();
        let expression_type: &Type = expression.get_value_type()?;

        let cast: Type = typegen::build_type(ctx, false)?;

        let is_constant: bool = expression.is_constant_value();
        let is_allocated: bool = expression.is_allocated() || expression_type.is_ptr_type();

        expression = Ast::As {
            from: expression.into(),
            cast,
            metadata: CastMetadata::new(is_constant, is_allocated),
            span,
        };
    }

    Ok(expression)
}
