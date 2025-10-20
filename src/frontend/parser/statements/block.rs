use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, statement},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_block<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    let block_tk: &Token = ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let span: Span = block_tk.get_span();

    ctx.begin_scope();
    ctx.get_mut_symbols().begin_scope();

    let mut stmts: Vec<Ast> = Vec::with_capacity(256);

    while !ctx.match_token(TokenType::RBrace)? {
        let stmt: Ast = statement::parse(ctx)?;
        stmts.push(stmt)
    }

    ctx.get_mut_symbols().end_scope();
    ctx.end_scope();

    Ok(Ast::Block { stmts, span })
}

pub fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
