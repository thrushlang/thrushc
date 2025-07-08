use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_global_assembler<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_double_global_assembler_state(parser_context)?;

    let glasm_keyword_tk: &Token = parser_context.consume(
        TokenType::GlobalAsm,
        "Syntax error".into(),
        "Expected 'glasm' keyword.".into(),
    )?;

    let span: Span = glasm_keyword_tk.get_span();

    parser_context.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let assembler: Ast = expr::build_expr(parser_context)?;
    let asssembler_span: Span = assembler.get_span();

    parser_context.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    parser_context.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    parser_context.get_mut_control_ctx().set_global_asm(true);

    let asm: &str = assembler.get_str_content(asssembler_span)?;

    Ok(Ast::GlobalAssembler {
        asm: asm.to_string(),
        span,
    })
}
