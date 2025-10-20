use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::term},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn cmp_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = term::term_precedence(ctx)?;

    if ctx.match_token(TokenType::Greater)?
        || ctx.match_token(TokenType::GreaterEq)?
        || ctx.match_token(TokenType::Less)?
        || ctx.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = term::term_precedence(ctx)?;

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
