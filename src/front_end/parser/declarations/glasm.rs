use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstStandardExtensions};
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

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

    if !assembler.is_str() {
        ctx.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Expected string literal value.".into(),
            None,
            asssembler_span,
        ));
    }

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

    let asm: String = if let Ast::Str { bytes, .. } = assembler {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };

    Ok(Ast::GlobalAssembler { asm, span })
}
