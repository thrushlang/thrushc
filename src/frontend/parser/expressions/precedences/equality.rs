use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::cast},
        types::{ast::Ast, lexer::Type, parser::stmts::traits::TokenExtensions},
    },
};

pub fn equality_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = cast::cast_precedence(parser_context)?;

    if parser_context.match_token(TokenType::BangEq)?
        || parser_context.match_token(TokenType::EqEq)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: Ast = cast::cast_precedence(parser_context)?;

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
