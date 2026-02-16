use thrustc_ast::Ast;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::ParserContext;

pub fn build_import<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Import,
        CompilationIssueCode::E0001,
        "Expected 'import' keyword.".into(),
    )?;

    let path_literal_tk: &Token = ctx.consume_these(
        &[TokenType::CString, TokenType::CNString],
        CompilationIssueCode::E0001,
        "Expected string literal.".into(),
    )?;

    let span: Span = path_literal_tk.get_span();

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
