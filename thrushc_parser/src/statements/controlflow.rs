use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::ParserContext;

pub fn build_continue<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let continue_tk: &Token = ctx.consume(
        TokenType::Continue,
        CompilationIssueCode::E0001,
        "Expected 'continue' keyword.".into(),
    )?;

    let span: Span = continue_tk.span;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Continue {
        span,
        kind: Type::Void(span),
    })
}

pub fn build_break<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let break_tk: &Token = ctx.consume(
        TokenType::Break,
        CompilationIssueCode::E0001,
        "Expected 'break' keyword.".into(),
    )?;

    let span: Span = break_tk.get_span();

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Break {
        span,
        kind: Type::Void(span),
    })
}
