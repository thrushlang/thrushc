use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::reference, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_sizeof<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::SizeOf,
        String::from("Syntax error"),
        String::from("Expected 'sizeof' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let sizeof_span: Span = sizeof_tk.get_span();

    if ctx.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = ctx.previous();

        let name: &str = identifier_tk.get_lexeme();
        let span: Span = identifier_tk.get_span();

        let reference: Ast = reference::build_reference(ctx, name, span)?;

        let reference_type: &Type = reference.get_value_type()?;

        ctx.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        return Ok(Ast::SizeOf {
            sizeof: reference_type.clone(),
            kind: Type::U32,
            span: sizeof_span,
        });
    }

    let sizeof_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::SizeOf {
        sizeof: sizeof_type,
        kind: Type::U32,
        span: sizeof_span,
    })
}
