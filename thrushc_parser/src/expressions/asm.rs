use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstStandardExtensions},
};
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, expressions, typegen};

pub fn build_asm_code_block<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let asm_tk: &Token = ctx.consume(
        TokenType::Asm,
        CompilationIssueCode::E0001,
        "Expected 'asm' keyword.".into(),
    )?;

    let asm_type: Type = typegen::build_type(ctx, false)?;

    let span: Span = asm_tk.get_span();

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::LParen, TokenType::LBrace])?;

    if ctx.match_token(TokenType::LParen)? {
        loop {
            if ctx.check(TokenType::RParen) {
                break;
            }

            args.push(expressions::build_expression(ctx)?);

            if ctx.check(TokenType::RParen) {
                break;
            } else {
                ctx.consume(
                    TokenType::Colon,
                    CompilationIssueCode::E0001,
                    "Expected ','.".into(),
                )?;
            }
        }

        ctx.consume(
            TokenType::RParen,
            CompilationIssueCode::E0001,
            "Expected ')'.".into(),
        )?;
    }

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expressions::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let assembly: String = if let Ast::Str { bytes, .. } = raw_str {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            String::new()
        };

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(&assembly);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }

        assembler_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
        "Expected '}'.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expressions::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let constraint: String = if let Ast::Str { bytes, .. } = raw_str {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            String::new()
        };

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(&constraint);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }

        constraint_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
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
