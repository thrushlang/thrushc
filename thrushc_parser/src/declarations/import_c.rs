use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::ParserContext;

pub fn build_import_c<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::ImportC,
        CompilationIssueCode::E0001,
        "Expected 'importC' keyword.".into(),
    )?;

    let span: Span = tk.get_span();

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

    Ok(Ast::ImportC {
        span,
        kind: Type::Void(span),
    })
}
