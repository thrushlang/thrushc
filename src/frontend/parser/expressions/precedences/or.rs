use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::and},
        types::{ast::Ast, lexer::Type, parser::stmts::traits::TokenExtensions},
    },
};

pub fn or_precedence<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<Ast<'instr>, ThrushCompilerIssue> {
    let mut expression: Ast = and::and_precedence(parser_context)?;

    while parser_context.match_token(TokenType::Or)? {
        let operator_tk: &Token = parser_context.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: Ast = and::and_precedence(parser_context)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool,
            span,
        }
    }

    Ok(expression)
}
