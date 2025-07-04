use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_global_assembler<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let glasm_keyword_tk: &Token = parser_context.consume(
        TokenType::Glasm,
        "Syntax error".into(),
        "Expected 'glasm' keyword.".into(),
    )?;

    let span: Span = glasm_keyword_tk.get_span();

    if parser_context.get_control_ctx().get_global_asm() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Duplicated global assembler.".into(),
            None,
            glasm_keyword_tk.get_span(),
        ));
    }

    parser_context.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let assembler: Ast = expression::build_expr(parser_context)?;
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
