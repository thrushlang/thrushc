use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::cmp, typegen},
        types::{
            ast::{Ast, metadata::cast::CastMetadata},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::types::Type,
    },
};

pub fn cast_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = cmp::cmp_precedence(parser_context)?;

    if parser_context.match_token(TokenType::As)? {
        let span: Span = parser_context.previous().get_span();

        let cast: Type = typegen::build_type(parser_context)?;

        let is_constant: bool = expression.is_constant_value();

        expression = Ast::As {
            from: expression.into(),
            cast,
            metadata: CastMetadata::new(is_constant),
            span,
        };
    }

    Ok(expression)
}
