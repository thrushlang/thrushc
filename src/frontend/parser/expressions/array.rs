use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::{traits::TypeArrayEntensions, types::Type},
    },
};

pub fn build_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let array_start_tk: &Token = ctx.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: Type = Type::Void;
    let mut items: Vec<Ast> = Vec::with_capacity(100);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let item: Ast = expr::build_expr(ctx)?;

        items.push(item);

        if ctx.check(TokenType::RBracket) {
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
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    if let Some(item) = items.iter().max_by(|a, b| {
        let a_type: &Type = a.get_value_type().unwrap_or(&Type::Void);
        let b_type: &Type = b.get_value_type().unwrap_or(&Type::Void);

        a_type
            .get_array_type_herarchy()
            .cmp(&b_type.get_array_type_herarchy())
    }) {
        array_type = Type::Array(item.get_value_type()?.clone().into())
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
    })
}
