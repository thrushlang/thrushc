use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstStandardExtensions},
};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions};

pub fn build_global_assembler<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::GlobalAsm,
        CompilationIssueCode::E0001,
        "Expected 'global_asm' keyword.".into(),
    )?;

    let span: Span = tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let assembler: Ast = expressions::build_expr(ctx)?;
    let asssembler_span: Span = assembler.get_span();

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    if !assembler.is_cnstring() {
        ctx.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Expected string literal value with null termination.".into(),
            None,
            asssembler_span,
        ));
    }

    let asm: String = if let Ast::CString { bytes, .. } = assembler {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };

    Ok(Ast::GlobalAssembler {
        asm,
        span,
        kind: Type::Void(span),
    })
}
