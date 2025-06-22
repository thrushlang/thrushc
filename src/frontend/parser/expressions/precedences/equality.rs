use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::cast},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn equality_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = cast::cast_precedence(parser_context)?;

    if parser_context.match_token(TokenType::BangEq)?
        || parser_context.match_token(TokenType::EqEq)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = cast::cast_precedence(parser_context)?;

        expression = ThrushStatement::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        }
    }

    Ok(expression)
}
