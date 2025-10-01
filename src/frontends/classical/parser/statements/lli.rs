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
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Instr,
        String::from("Syntax error"),
        String::from("Expected 'instr' keyword."),
    )?;

    let instr_tk: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let name: &str = instr_tk.get_lexeme();
    let span: Span = instr_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let instr_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let expr: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    ctx.get_mut_symbols()
        .new_lli(name, (instr_type.clone(), span), span)?;

    Ok(Ast::LLI {
        name,
        kind: instr_type,
        expr: expr.into(),
        span,
    })
}

fn check_state(ctx: &mut ParserContext<'_>) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
