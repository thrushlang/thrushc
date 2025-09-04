use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, statement},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_block<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    let block_tk: &Token = parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let span: Span = block_tk.get_span();

    *parser_context.get_mut_scope() += 1;
    parser_context.get_mut_symbols().begin_scope();

    let mut stmts: Vec<Ast> = Vec::with_capacity(256);

    while !parser_context.match_token(TokenType::RBrace)? {
        let stmt: Ast = statement::parse(parser_context)?;
        stmts.push(stmt)
    }

    parser_context.get_mut_symbols().end_scope();
    *parser_context.get_mut_scope() -= 1;

    Ok(Ast::Block { stmts, span })
}

pub fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
