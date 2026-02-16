use thrustc_ast::{Ast, traits::AstStandardExtensions};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

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

    let mut nodes: Vec<Ast> = Vec::with_capacity(u8::MAX as usize);
    let mut post: Vec<Ast> = Vec::with_capacity(32);

    while !ctx.match_token(TokenType::RBrace)? {
        let statement: Ast<'_> = statements::parse(ctx)?;

        if statement.is_post_execution_at_scope() {
            post.push(statement);
        } else {
            nodes.push(statement);
        }
    }

    ctx.get_mut_symbols().end_scope();
    ctx.end_scope();

    Ok(Ast::Block {
        nodes,
        post,
        span,
        kind: Type::Void(span),
    })
}

pub fn build_block_without_start<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.begin_scope();
    ctx.get_mut_symbols().begin_scope();

    let mut nodes: Vec<Ast> = Vec::with_capacity(u8::MAX as usize);
    let mut post: Vec<Ast> = Vec::with_capacity(32);

    while !ctx.check(TokenType::RBrace) {
        let statement: Ast<'_> = statements::parse(ctx)?;

        if statement.is_post_execution_at_scope() {
            post.push(statement);
        } else {
            nodes.push(statement);
        }
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
        post,
        span,
        kind: Type::Void(span),
    })
}
