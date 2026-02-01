use thrushc_ast::{Ast, traits::AstCodeBlockEntensions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions, statements::block};

pub fn build_conditional<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let if_tk: &Token = ctx.consume(
        TokenType::If,
        CompilationIssueCode::E0001,
        "Expected 'if' keyword.".into(),
    )?;

    let span: Span = if_tk.get_span();

    let condition: Ast = expressions::build_expr(ctx)?;
    let block: Ast = block::build_block(ctx)?;

    let mut elseif: Vec<Ast> = Vec::with_capacity(10);

    while ctx.match_token(TokenType::Elif)?
        || (ctx.match_token(TokenType::Else)? && ctx.match_token(TokenType::If)?)
    {
        let span: Span = ctx.previous().get_span();

        let condition: Ast = expressions::build_expr(ctx)?;
        let else_if_block: Ast = block::build_block(ctx)?;

        if !else_if_block.is_empty_block() {
            elseif.push(Ast::Elif {
                condition: condition.into(),
                block: else_if_block.into(),
                kind: Type::Void(span),
                span,
            });
        }
    }

    if ctx.match_token(TokenType::Else)? {
        let span: Span = ctx.previous().get_span();
        let else_block: Ast = block::build_block(ctx)?;

        if !else_block.is_empty_block() {
            let else_node: Ast = Ast::Else {
                block: else_block.into(),
                kind: Type::Void(span),
                span,
            };

            return Ok(Ast::If {
                condition: condition.into(),
                block: block.into(),
                elseif,
                anyway: Some(else_node.into()),
                kind: Type::Void(span),
                span,
            });
        }
    }

    Ok(Ast::If {
        condition: condition.into(),
        block: block.into(),
        elseif,
        anyway: None,
        kind: Type::Void(span),
        span,
    })
}
