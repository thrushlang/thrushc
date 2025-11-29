use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::checks;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_import<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
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
