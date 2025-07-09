use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::equality},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn and_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = equality::equality_precedence(parser_context)?;

    while parser_context.match_token(TokenType::And)? {
        let operator_tk: &Token = parser_context.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Ast = equality::equality_precedence(parser_context)?;

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
