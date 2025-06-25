use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{ast::Ast, lexer::ThrushType, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_array<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let array_start_tk: &Token = parser_context.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: ThrushType = ThrushType::Void;
    let mut items: Vec<Ast> = Vec::with_capacity(100);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let item: Ast = expression::build_expr(parser_context)?;

        if item.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                item.get_span(),
            ));
        }

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
        let a_type: &ThrushType = a.get_value_type().unwrap_or(&ThrushType::Void);
        let b_type: &ThrushType = b.get_value_type().unwrap_or(&ThrushType::Void);

        a_type
            .get_array_type_herarchy()
            .cmp(&b_type.get_array_type_herarchy())
    }) {
        array_type = ThrushType::Array(item.get_value_type()?.clone().into())
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
    })
}
