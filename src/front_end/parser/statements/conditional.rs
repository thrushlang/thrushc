use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::checks;
use crate::front_end::parser::expr;
use crate::front_end::parser::statements::block;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeBlockEntensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_conditional<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    self::check_state(ctx)?;

    let if_tk: &Token = ctx.consume(
        TokenType::If,
        "Syntax error".into(),
        "Expected 'if' keyword.".into(),
    )?;

    let span: Span = if_tk.get_span();

    let condition: Ast = expr::build_expr(ctx)?;
    let block: Ast = block::build_block(ctx)?;

    let mut elseif: Vec<Ast> = Vec::with_capacity(10);

    while ctx.match_token(TokenType::Elif)? {
        let span: Span = ctx.previous().get_span();

        let condition: Ast = expr::build_expr(ctx)?;
        let else_if_block: Ast = block::build_block(ctx)?;

        if !else_if_block.is_empty_block() {
            elseif.push(Ast::Elif {
                condition: condition.into(),
                block: else_if_block.into(),
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
                span,
            };

            return Ok(Ast::If {
                condition: condition.into(),
                block: block.into(),
                elseif,
                anyway: Some(else_node.into()),
                span,
            });
        }
    }

    Ok(Ast::If {
        condition: condition.into(),
        block: block.into(),
        elseif,
        anyway: None,
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), CompilationIssue> {
    checks::check_inside_function_state(ctx)
}
