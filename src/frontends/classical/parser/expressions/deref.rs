use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::ast::Ast,
        typesystem::{traits::DereferenceExtensions, types::Type},
    },
};

pub fn build_dereference<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let initial_deref_tk: &Token = ctx.advance()?;
    let span: Span = initial_deref_tk.span;

    let mut deref_count: u64 = 1;

    let mut current_expr: Ast = {
        while ctx.check(TokenType::Deref) {
            ctx.consume(
                TokenType::Deref,
                "Syntax error".into(),
                "Expected 'deref' keyword.".into(),
            )?;

            deref_count += 1;
        }

        let expr: Ast = expr::build_expr(ctx)?;

        expr
    };

    let mut current_type: Type = current_expr.get_value_type()?.clone();

    (0..deref_count).for_each(|_| {
        current_expr = Ast::Deref {
            value: current_expr.clone().into(),
            kind: current_type.dereference(),
            span,
        };

        current_type = current_type.dereference();
    });

    Ok(current_expr)
}
