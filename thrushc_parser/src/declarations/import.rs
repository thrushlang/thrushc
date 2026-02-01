use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::ParserContext;

pub fn build_import<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let import_tk: &Token = ctx.consume(
        TokenType::Import,
        CompilationIssueCode::E0001,
        "Expected 'import' keyword.".into(),
    )?;

    let span: Span = import_tk.get_span();

    ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected string literal.".into(),
    )?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Import {
        span,
        kind: Type::Void(span),
    })
}
