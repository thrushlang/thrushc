use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_global_assembler<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_double_global_assembler_state(ctx)?;

    let glasm_keyword_tk: &Token = ctx.consume(
        TokenType::GlobalAsm,
        "Syntax error".into(),
        "Expected 'glasm' keyword.".into(),
    )?;

    let span: Span = glasm_keyword_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let assembler: Ast = expr::build_expr(ctx)?;
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

    ctx.get_mut_control_ctx().set_global_asm(true);

    let asm: &str = assembler.get_str_content(asssembler_span)?;

    Ok(Ast::GlobalAssembler {
        asm: asm.to_string(),
        span,
    })
}
