use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, stmt},
        types::parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
    },
};

pub fn build_block<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let block_tk: &Token = parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let span: Span = block_tk.get_span();

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Block of code must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    *parser_ctx.get_mut_scope() += 1;
    parser_ctx.get_mut_symbols().begin_scope();

    let mut stmts: Vec<ThrushStatement> = Vec::with_capacity(100);

    while !parser_ctx.match_token(TokenType::RBrace)? {
        let stmt: ThrushStatement = stmt::statement(parser_ctx)?;
        stmts.push(stmt)
    }

    parser_ctx.get_mut_symbols().end_scope();
    *parser_ctx.get_mut_scope() -= 1;

    Ok(ThrushStatement::Block { stmts, span })
}
