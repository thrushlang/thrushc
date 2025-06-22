use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::lower},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn factor<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = lower::lower_precedence(parser_context)?;

    while parser_context.match_token(TokenType::Slash)?
        || parser_context.match_token(TokenType::Star)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = lower::lower_precedence(parser_context)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_numeric_type(right_type);

        expression = ThrushStatement::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: kind.clone(),
            span,
        };
    }

    Ok(expression)
}
