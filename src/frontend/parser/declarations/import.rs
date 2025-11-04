use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::checks;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;

pub fn build_import<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    let import_tk: &Token = ctx.consume(
        TokenType::Import,
        "Syntax error".into(),
        "Expected 'import' keyword.".into(),
    )?;

    let span: Span = import_tk.get_span();

    ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected string literal.".into(),
    )?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Import { span })
}
