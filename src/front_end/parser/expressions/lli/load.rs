use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions, typegen};
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn build_load<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::Load,
        "Syntax error".into(),
        "Expected 'load' keyword.".into(),
    )?;

    let span: Span = tk.get_span();

    let load_type: Type = typegen::build_type(ctx, false)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let source: Ast = expressions::build_expr(ctx)?;

    Ok(Ast::Load {
        source: source.into(),
        kind: load_type,
        span,
    })
}
