use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::comparation, typegen},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn cast_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = comparation::cmp_precedence(parser_context)?;

    if parser_context.match_token(TokenType::As)? {
        let span: Span = parser_context.previous().get_span();

        let cast: ThrushType = typegen::build_type(parser_context)?;

        expression = ThrushStatement::As {
            from: expression.into(),
            cast,
            span,
        };
    }

    Ok(expression)
}
