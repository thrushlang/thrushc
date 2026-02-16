use thrustc_ast::Ast;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{
    ParserContext, expressions,
    statements::{self, block, local},
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

    if ctx.check(TokenType::LBrace) {
        let body: Ast<'_> = block::build_block(ctx)?;

        Ok(Ast::Loop {
            block: body.into(),
            kind: Type::Void(span),
            span,
        })
    } else if ctx.match_token(TokenType::SemiColon)? {
        while ctx.match_token(TokenType::SemiColon)? {}

        let body: Ast<'_> = if ctx.check(TokenType::LBrace) {
            block::build_block(ctx)?
        } else {
            statements::parse(ctx)?
        };

        Ok(Ast::Loop {
            block: body.into(),
            kind: Type::Void(span),
            span,
        })
    } else {
        ctx.get_mut_symbols().begin_scope();
        ctx.begin_scope();

        let local: Ast = local::build_local(ctx)?;
        let condition: Ast = expressions::build_expression(ctx)?;
        let actions: Ast = expressions::build_expression(ctx)?;

        let body: Ast = if ctx.check(TokenType::LBrace) {
            block::build_block(ctx)?
        } else {
            statements::parse(ctx)?
        };

        ctx.get_mut_symbols().end_scope();
        ctx.end_scope();

        Ok(Ast::For {
            local: local.into(),
            condition: condition.into(),
            actions: actions.into(),
            block: body.into(),
            kind: Type::Void(span),
            span,
        })
    }
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
    let body: Ast = if ctx.check(TokenType::LBrace) {
        block::build_block(ctx)?
    } else {
        statements::parse(ctx)?
    };

    Ok(Ast::Loop {
        block: body.into(),
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

    if ctx.check(TokenType::Local) {
        ctx.get_mut_symbols().begin_scope();
        ctx.begin_scope();

        let local: Ast<'_> = local::build_local(ctx)?;

        ctx.consume(
            TokenType::Colon,
            CompilationIssueCode::E0001,
            "Expected ':'.".into(),
        )?;

        let condition: Ast = expressions::build_expr(ctx)?;
        let body: Ast = if ctx.check(TokenType::LBrace) {
            block::build_block(ctx)?
        } else {
            statements::parse(ctx)?
        };

        ctx.get_mut_symbols().end_scope();
        ctx.end_scope();

        Ok(Ast::While {
            variable: Some(local.into()),
            condition: condition.into(),
            block: body.into(),
            kind: Type::Void(span),
            span,
        })
    } else if ctx.check(TokenType::LParen) {
        let mut found_rparen: usize = 0;

        while ctx.match_token(TokenType::LParen)? {
            found_rparen += 1;
        }

        if ctx.check(TokenType::Local) {
            ctx.get_mut_symbols().begin_scope();
            ctx.begin_scope();

            let local: Ast<'_> = local::build_local(ctx)?;

            ctx.consume(
                TokenType::Colon,
                CompilationIssueCode::E0001,
                "Expected ':'.".into(),
            )?;

            let condition: Ast = expressions::build_expr(ctx)?;

            for _ in 0..=found_rparen {
                ctx.consume(
                    TokenType::RParen,
                    CompilationIssueCode::E0001,
                    "Expected ')'.".into(),
                )?;
            }

            let body: Ast = if ctx.check(TokenType::LBrace) {
                block::build_block(ctx)?
            } else {
                statements::parse(ctx)?
            };

            ctx.get_mut_symbols().end_scope();
            ctx.end_scope();

            Ok(Ast::While {
                variable: Some(local.into()),
                condition: condition.into(),
                block: body.into(),
                kind: Type::Void(span),
                span,
            })
        } else {
            let condition: Ast = expressions::build_expr(ctx)?;

            for _ in 0..=found_rparen {
                ctx.consume(
                    TokenType::RParen,
                    CompilationIssueCode::E0001,
                    "Expected ')'.".into(),
                )?;
            }

            let block: Ast = block::build_block(ctx)?;

            Ok(Ast::While {
                variable: None,
                condition: condition.into(),
                block: block.into(),
                kind: Type::Void(span),
                span,
            })
        }
    } else {
        let condition: Ast = expressions::build_expr(ctx)?;
        let block: Ast = block::build_block(ctx)?;

        Ok(Ast::While {
            variable: None,
            condition: condition.into(),
            block: block.into(),
            kind: Type::Void(span),
            span,
        })
    }
}
