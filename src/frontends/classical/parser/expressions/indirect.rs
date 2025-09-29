use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn build_indirect<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::Indirect,
        String::from("Syntax error"),
        String::from("Expected 'indirect' keyword."),
    )?;

    let span: Span = ctx.previous().get_span();

    let expression: Ast = expr::build_expr(ctx)?;
    let expression_type: &Type = expression.get_value_type()?;

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let expression: Ast = expr::build_expr(ctx)?;

        args.push(expression);

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Indirect {
        pointer: expression.clone().into(),
        function_type: expression_type.clone(),
        args,
        kind: expression_type.get_type_fn_ref().clone(),
        span,
    })
}
