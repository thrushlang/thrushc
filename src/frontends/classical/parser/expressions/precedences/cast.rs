use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::mutation, typegen},
        types::{
            ast::{Ast, metadata::cast::CastMetadata, traits::LLVMAstExtensions},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::types::Type,
    },
};

pub fn cast_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = mutation::equal_precedence(ctx)?;

    if ctx.match_token(TokenType::As)? {
        let span: Span = ctx.previous().get_span();
        let expression_type: &Type = expression.get_value_type()?;

        let cast: Type = typegen::build_type(ctx)?;

        let is_constant: bool = expression.is_llvm_constant_value();

        let is_allocated: bool = expression.is_allocated()
            || expression_type.is_mut_type()
            || expression_type.is_ptr_type();

        expression = Ast::As {
            from: expression.into(),
            cast,
            metadata: CastMetadata::new(is_constant, is_allocated),
            span,
        };
    }

    Ok(expression)
}
