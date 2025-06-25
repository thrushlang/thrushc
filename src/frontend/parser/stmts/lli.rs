use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression, typegen},
        types::{ast::Ast, lexer::ThrushType, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_lli<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let instr_tk: &Token = parser_ctx.consume(
        TokenType::Instr,
        String::from("Syntax error"),
        String::from("Expected 'instr' keyword."),
    )?;

    let span: Span = instr_tk.get_span();

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("LLI's should be contained at local scope."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let instr_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let name: &str = instr_tk.get_lexeme();
    let span: Span = instr_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let instr_type: ThrushType = typegen::build_type(parser_ctx)?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    parser_ctx
        .get_mut_symbols()
        .new_lli(name, (instr_type.clone(), span), span)?;

    let lli: Ast = Ast::LLI {
        name,
        kind: instr_type,
        value: value.into(),
        span,
    };

    Ok(lli)
}
