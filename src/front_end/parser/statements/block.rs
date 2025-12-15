use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, statements};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_block<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let block_tk: &Token = ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let span: Span = block_tk.get_span();

    ctx.begin_scope();
    ctx.get_mut_symbols().begin_scope();

    let mut nodes: Vec<Ast> = Vec::with_capacity(256);

    while !ctx.match_token(TokenType::RBrace)? {
        nodes.push(statements::parse(ctx)?)
    }

    ctx.get_mut_symbols().end_scope();
    ctx.end_scope();

    Ok(Ast::Block { nodes, span })
}
