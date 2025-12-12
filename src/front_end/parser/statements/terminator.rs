use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_return<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let return_tk: &Token = ctx.consume(
        TokenType::Return,
        "Syntax error".into(),
        "Expected 'return' keyword.".into(),
    )?;

    let span: Span = return_tk.get_span();

    if ctx.match_token(TokenType::SemiColon)? {
        return Ok(Ast::Return {
            expression: None,
            kind: Type::Void,
            span,
        });
    }

    let value: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Return {
        expression: Some(value.into()),
        kind: ctx.get_type_ctx().get_function_type().clone(),
        span,
    })
}
