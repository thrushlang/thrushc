use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::and},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn or_precedence<'instr>(
    ctx: &mut ParserContext<'instr>,
) -> Result<Ast<'instr>, ThrushCompilerIssue> {
    let mut expression: Ast = and::and_precedence(ctx)?;

    while ctx.match_token(TokenType::Or)? {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: Ast = and::and_precedence(ctx)?;

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
