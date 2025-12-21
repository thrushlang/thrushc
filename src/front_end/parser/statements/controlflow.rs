use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

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

    Ok(Ast::Continue { span })
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

    Ok(Ast::Break { span })
}
