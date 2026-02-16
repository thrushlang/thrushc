use thrustc_ast::Ast;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::ParserContext;

pub fn build_continue<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let continue_tk: &Token = ctx.consume(
        TokenType::Continue,
        CompilationIssueCode::E0001,
        "Expected 'continue' keyword.".into(),
    )?;

    let span: Span = continue_tk.get_span();

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

pub fn build_continueall<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let continueall_tk: &Token = ctx.consume(
        TokenType::ContinueAll,
        CompilationIssueCode::E0001,
        "Expected 'continueall' keyword.".into(),
    )?;

    let span: Span = continueall_tk.get_span();

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::ContinueAll {
        span,
        kind: Type::Void(span),
    })
}

pub fn build_breakall<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let breakall_tk: &Token = ctx.consume(
        TokenType::BreakAll,
        CompilationIssueCode::E0001,
        "Expected 'breakall' keyword.".into(),
    )?;

    let span: Span = breakall_tk.get_span();

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::BreakAll {
        span,
        kind: Type::Void(span),
    })
}
