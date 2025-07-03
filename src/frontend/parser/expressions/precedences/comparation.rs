use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::term},
        types::{ast::Ast, lexer::Type, parser::stmts::traits::TokenExtensions},
    },
};

pub fn cmp_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = term::term_precedence(parser_context)?;

    if parser_context.match_token(TokenType::Greater)?
        || parser_context.match_token(TokenType::GreaterEq)?
        || parser_context.match_token(TokenType::Less)?
        || parser_context.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = parser_context.previous();

        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = term::term_precedence(parser_context)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool,
            span,
        };
    }

    Ok(expression)
}
