use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::ast::Ast,
        types::lexer::ThrushType,
    },
};

pub fn build_dereference<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let initial_deref_tk: &Token = parser_context.advance()?;
    let span: Span = initial_deref_tk.span;

    let mut deref_count: u64 = 1;

    let mut current_expr: Ast = {
        while parser_context.check(TokenType::Deref) {
            parser_context.consume(
                TokenType::Deref,
                "Syntax error".into(),
                "Expected 'deref'.".into(),
            )?;

            deref_count += 1;
        }

        let expr: Ast = expression::build_expr(parser_context)?;

        expr
    };

    let mut current_type: ThrushType = current_expr.get_value_type()?.clone();

    (0..deref_count).for_each(|_| {
        current_expr = Ast::Deref {
            value: current_expr.clone().into(),
            kind: current_type.deref(),
            span,
        };

        current_type = current_type.deref();
    });

    Ok(current_expr)
}
