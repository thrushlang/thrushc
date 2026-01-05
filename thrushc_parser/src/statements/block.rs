use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, statements};

pub fn build_block<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let block_tk: &Token = ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
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

    Ok(Ast::Block {
        nodes,
        span,
        kind: Type::Void(span),
    })
}

pub fn build_block_without_start<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.begin_scope();
    ctx.get_mut_symbols().begin_scope();

    let mut nodes: Vec<Ast> = Vec::with_capacity(256);

    while !ctx.check(TokenType::RBrace) {
        nodes.push(statements::parse(ctx)?)
    }

    let block_tk: &Token = ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
        "Expected '}'.".into(),
    )?;

    let span: Span = block_tk.get_span();

    ctx.get_mut_symbols().end_scope();
    ctx.end_scope();

    Ok(Ast::Block {
        nodes,
        span,
        kind: Type::Void(span),
    })
}
