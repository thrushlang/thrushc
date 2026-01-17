use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{
    ParserContext, expressions,
    statements::{block, local},
};

pub fn build_for_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let for_tk: &Token = ctx.consume(
        TokenType::For,
        CompilationIssueCode::E0001,
        "Expected 'for' keyword.".into(),
    )?;

    let span: Span = for_tk.get_span();

    let local: Ast = local::build_local(ctx)?;
    let condition: Ast = expressions::build_expression(ctx)?;
    let actions: Ast = expressions::build_expression(ctx)?;
    let body: Ast = block::build_block(ctx)?;

    Ok(Ast::For {
        local: local.into(),
        condition: condition.into(),
        actions: actions.into(),
        block: body.into(),
        kind: Type::Void(span),
        span,
    })
}

pub fn build_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let loop_tk: &Token = ctx.consume(
        TokenType::Loop,
        CompilationIssueCode::E0001,
        "Expected 'loop' keyword.".into(),
    )?;

    let span: Span = loop_tk.get_span();

    let block: Ast = block::build_block(ctx)?;

    Ok(Ast::Loop {
        block: block.into(),
        kind: Type::Void(span),
        span,
    })
}

pub fn build_while_loop<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let while_tk: &Token = ctx.consume(
        TokenType::While,
        CompilationIssueCode::E0001,
        "Expected 'while' keyword.".into(),
    )?;

    let span: Span = while_tk.get_span();

    let condition: Ast = expressions::build_expr(ctx)?;
    let block: Ast = block::build_block(ctx)?;

    Ok(Ast::While {
        condition: condition.into(),
        block: block.into(),
        kind: Type::Void(span),
        span,
    })
}
