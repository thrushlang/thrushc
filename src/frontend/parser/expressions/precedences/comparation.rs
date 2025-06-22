use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::term},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn cmp_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = term::term_precedence(parser_context)?;

    if parser_context.match_token(TokenType::Greater)?
        || parser_context.match_token(TokenType::GreaterEq)?
        || parser_context.match_token(TokenType::Less)?
        || parser_context.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = parser_context.previous();

        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = term::term_precedence(parser_context)?;

        expression = ThrushStatement::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        };
    }

    Ok(expression)
}
