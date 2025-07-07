use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
        typesystem::types::Type,
    },
};

pub fn build_fixed_array<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_context.consume(
        TokenType::Fixed,
        String::from("Syntax error"),
        String::from("Expected 'fixed' keyword."),
    )?;

    let array_start_tk: &Token = parser_context.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: Type = Type::Void;
    let mut items: Vec<Ast> = Vec::with_capacity(100);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let item: Ast = expression::build_expr(parser_context)?;

        items.push(item);

        if parser_context.check(TokenType::RBracket) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
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
        if let Ok(size) = u32::try_from(items.len()) {
            array_type = Type::FixedArray(item.get_value_type()?.clone().into(), size)
        } else {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "The size limit of an array was exceeded.".into(),
                None,
                span,
            ));
        }
    }

    Ok(Ast::FixedArray {
        items,
        kind: array_type,
        span,
    })
}
