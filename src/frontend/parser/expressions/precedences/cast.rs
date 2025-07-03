use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::comparation, typegen},
        types::{ast::Ast, lexer::Type, parser::stmts::traits::TokenExtensions},
    },
};

pub fn cast_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = comparation::cmp_precedence(parser_context)?;

    if parser_context.match_token(TokenType::As)? {
        let span: Span = parser_context.previous().get_span();

        let cast: Type = typegen::build_type(parser_context)?;

        expression = Ast::As {
            from: expression.into(),
            cast,
            span,
        };
    }

    Ok(expression)
}
