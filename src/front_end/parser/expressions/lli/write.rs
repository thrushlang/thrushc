use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions, typegen};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_write<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let write_tk: &Token = ctx.consume(
        TokenType::Write,
        CompilationIssueCode::E0001,
        "Expected 'write' keyword.".into(),
    )?;

    let span: Span = write_tk.get_span();

    let source: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        "Expected ','.".into(),
    )?;

    let write_type: Type = typegen::build_type(ctx, false)?;
    let value: Ast = expressions::build_expr(ctx)?;

    Ok(Ast::Write {
        source: source.into(),
        write_value: value.clone().into(),
        write_type,
        span,
    })
}
