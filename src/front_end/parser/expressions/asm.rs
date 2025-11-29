use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, attributes, expr, typegen};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub fn build_asm_code_block<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let asm_tk: &Token = ctx.consume(
        TokenType::Asm,
        "Syntax error".into(),
        "Expected 'asm' keyword.".into(),
    )?;

    let asm_type: Type = typegen::build_type(ctx)?;

    let span: Span = asm_tk.get_span();

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::LParen, TokenType::LBrace])?;

    if ctx.match_token(TokenType::LParen)? {
        loop {
            if ctx.check(TokenType::RParen) {
                break;
            }

            let expr: Ast = expr::build_expression(ctx)?;

            args.push(expr);

            if ctx.check(TokenType::RParen) {
                break;
            } else {
                ctx.consume(
                    TokenType::Colon,
                    "Syntax error".into(),
                    "Expected ','.".into(),
                )?;
            }
        }

        ctx.consume(
            TokenType::RParen,
            "Syntax error".into(),
            "Expected ')'.".into(),
        )?;
    }

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expr::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            return Err(CompilationIssue::Error(
                "Syntax error".into(),
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let assembly: &str = raw_str.get_str_literal_content(raw_str_span)?;

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(assembly);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }

        assembler_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expr::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        let constraint: &str = raw_str.get_str_literal_content(raw_str_span)?;

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(constraint);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }

        constraint_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    Ok(Ast::AsmValue {
        assembler,
        constraints,
        args,
        kind: asm_type,
        attributes,
        span,
    })
}
