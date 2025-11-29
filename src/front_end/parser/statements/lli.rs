use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::checks;
use crate::front_end::parser::expr;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_lli<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
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

fn check_state(ctx: &mut ParserContext<'_>) -> Result<(), CompilationIssue> {
    checks::check_inside_function_state(ctx)
}
