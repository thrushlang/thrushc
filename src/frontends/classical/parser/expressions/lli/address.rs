use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_address<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let address_tk: &Token = ctx.advance()?;
    let address_span: Span = address_tk.get_span();

    if ctx.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = ctx.previous();

        let name: &str = identifier_tk.get_lexeme();
        let span: Span = identifier_tk.get_span();

        let reference: Ast = reference::build_reference(ctx, name, span)?;

        let indexes: Vec<Ast> = self::build_address_indexes(ctx, span)?;

        return Ok(Ast::Address {
            source: (Some((name, reference.into())), None),
            indexes,
            kind: Type::Addr,
            span: address_span,
        });
    }

    let expr: Ast = expr::build_expr(ctx)?;
    let expr_span: Span = expr.get_span();

    let indexes: Vec<Ast> = self::build_address_indexes(ctx, expr_span)?;

    Ok(Ast::Address {
        source: (None, Some(expr.into())),
        indexes,
        kind: Type::Addr,
        span: address_span,
    })
}

fn build_address_indexes<'parser>(
    ctx: &mut ParserContext<'parser>,
    span: Span,
) -> Result<Vec<Ast<'parser>>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut indexes: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let index: Ast = expr::build_expr(ctx)?;

        indexes.push(index);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if indexes.is_empty() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "At least one index was expected.".into(),
            None,
            span,
        ));
    }

    Ok(indexes)
}
