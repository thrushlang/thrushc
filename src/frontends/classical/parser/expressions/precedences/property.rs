use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{
            ParserContext,
            expressions::{precedences::lower, property},
        },
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn property_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = lower::lower_precedence(parser_context)?;

    if parser_context.match_token(TokenType::Dot)? {
        let span: Span = parser_context.previous().get_span();

        expression =
            property::build_property(parser_context, (None, Some(expression.into())), span)?;
    }

    Ok(expression)
}
