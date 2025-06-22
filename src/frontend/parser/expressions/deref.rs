use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn build_dereference<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let initial_deref_tk: &Token = parser_context.advance()?;
    let span: Span = initial_deref_tk.span;

    let mut deref_count: u64 = 1;

    let mut current_expr: ThrushStatement = {
        while parser_context.check(TokenType::Deref) {
            parser_context.consume(
                TokenType::Deref,
                "Syntax error".into(),
                "Expected 'deref'.".into(),
            )?;

            deref_count += 1;
        }

        let expr: ThrushStatement = expression::build_expr(parser_context)?;

        expr
    };

    let mut current_type: ThrushType = current_expr.get_value_type()?.clone();

    (0..deref_count).for_each(|_| {
        current_expr = ThrushStatement::Deref {
            value: current_expr.clone().into(),
            kind: current_type.deref(),
            span,
        };

        current_type = current_type.deref();
    });

    Ok(current_expr)
}
