use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_global_assembler<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::GlobalAsm,
        "Syntax error".into(),
        "Expected 'global_asm' keyword.".into(),
    )?;

    let span: Span = tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let assembler: Ast = expressions::build_expr(ctx)?;
    let asssembler_span: Span = assembler.get_span();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    let asm: &str = assembler.get_str_literal_content(asssembler_span)?;

    Ok(Ast::GlobalAssembler {
        asm: asm.to_string(),
        span,
    })
}
