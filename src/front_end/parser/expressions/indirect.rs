use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::traits::FunctionReferenceExtensions;
use crate::front_end::typesystem::types::Type;

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
