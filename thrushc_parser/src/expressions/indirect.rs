use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::{Type, traits::FunctionReferenceExtensions};

use crate::{ParserContext, expressions};
pub fn build_indirect<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Indirect,
        CompilationIssueCode::E0001,
        "Expected 'indirect' keyword.".into(),
    )?;

    let span: Span = ctx.previous().get_span();

    let expression: Ast = expressions::build_expr(ctx)?;
    let expression_type: &Type = expression.get_value_type()?;

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let expr: Ast = expressions::build_expr(ctx)?;

        args.push(expr);

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Indirect {
        function: expression.clone().into(),
        function_type: expression_type.clone(),
        args,
        kind: expression_type.get_fn_ref_type().clone(),
        span,
    })
}
