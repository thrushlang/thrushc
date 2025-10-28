use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::expr;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::typesystem::traits::TypeExtensions;
use crate::frontend::typesystem::types::Type;

pub fn build_indirect<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::Indirect,
        "Syntax error".into(),
        "Expected 'indirect' keyword.".into(),
    )?;

    let span: Span = ctx.previous().get_span();

    let expression: Ast = expr::build_expr(ctx)?;
    let expression_type: &Type = expression.get_value_type()?;

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let expr: Ast = expr::build_expr(ctx)?;

        args.push(expr);

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Indirect {
        function: expression.clone().into(),
        function_type: expression_type.clone(),
        args,
        kind: expression_type.get_type_fn_ref().clone(),
        span,
    })
}
