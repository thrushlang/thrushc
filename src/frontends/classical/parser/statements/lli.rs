use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_lli<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_ctx)?;

    parser_ctx.consume(
        TokenType::Instr,
        String::from("Syntax error"),
        String::from("Expected 'instr' keyword."),
    )?;

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

    let instr_type: Type = typegen::build_type(parser_ctx)?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expr::build_expr(parser_ctx)?;

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

fn check_state(parser_ctx: &mut ParserContext<'_>) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_ctx)?;
    checks::check_inside_function_state(parser_ctx)
}
