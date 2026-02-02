use thrushc_ast::{Ast, traits::AstCodeBlockEntensions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{
    ParserContext, expressions,
    statements::{self, block},
};

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

    let body: Ast = if ctx.check(TokenType::LBrace) {
        block::build_block(ctx)?
    } else {
        statements::parse(ctx)?
    };

    let mut elseif: Vec<Ast> = Vec::with_capacity(10);

    while ctx.check(TokenType::Elif)
        || (ctx.check(TokenType::Else) && ctx.check_to(TokenType::If, 1))
    {
        if ctx.check(TokenType::Elif) {
            ctx.consume(
                TokenType::Elif,
                CompilationIssueCode::E0001,
                "Expected 'elif' keyword.".into(),
            )?;
        } else {
            ctx.consume(
                TokenType::If,
                CompilationIssueCode::E0001,
                "Expected 'if' keyword.".into(),
            )?;

            ctx.consume(
                TokenType::Else,
                CompilationIssueCode::E0001,
                "Expected 'else' keyword.".into(),
            )?;
        }

        let span: Span = ctx.previous().get_span();

        let condition: Ast = expressions::build_expr(ctx)?;

        let body: Ast = if ctx.check(TokenType::LBrace) {
            block::build_block(ctx)?
        } else {
            statements::parse(ctx)?
        };

        if !body.is_empty_block() {
            elseif.push(Ast::Elif {
                condition: condition.into(),
                block: body.into(),
                kind: Type::Void(span),
                span,
            });
        }
    }

    if ctx.match_token(TokenType::Else)? {
        let span: Span = ctx.previous().get_span();

        let else_body: Ast = if ctx.check(TokenType::LBrace) {
            block::build_block(ctx)?
        } else {
            statements::parse(ctx)?
        };

        if !else_body.is_empty_block() {
            let else_node: Ast = Ast::Else {
                block: else_body.into(),
                kind: Type::Void(span),
                span,
            };

            return Ok(Ast::If {
                condition: condition.into(),
                block: body.into(),
                elseif,
                anyway: Some(else_node.into()),
                kind: Type::Void(span),
                span,
            });
        }
    }

    Ok(Ast::If {
        condition: condition.into(),
        block: body.into(),
        elseif,
        anyway: None,
        kind: Type::Void(span),
        span,
    })
}
