use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::checks;
use crate::frontend::parser::expr;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::typesystem::types::Type;

pub fn build_lli<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Instr,
        "Syntax error".into(),
        "Expected 'instr' keyword.".into(),
    )?;

    let instr_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = instr_tk.get_lexeme();
    let span: Span = instr_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let instr_type: Type = typegen::build_type(ctx)?;

    ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let expr: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
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
