use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::unary},
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn and_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = unary::unary_precedence(parser_context)?;

    while parser_context.match_token(TokenType::And)? {
        let operator_tk: &Token = parser_context.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = unary::unary_precedence(parser_context)?;

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
