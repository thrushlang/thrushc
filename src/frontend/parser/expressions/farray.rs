use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn build_fixed_array<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    let mut array_type: ThrushType = ThrushType::Void;
    let mut items: Vec<ThrushStatement> = Vec::with_capacity(100);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let item: ThrushStatement = expression::build_expr(parser_context)?;

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
        if let Ok(size) = u32::try_from(items.len()) {
            array_type = ThrushType::FixedArray(item.get_value_type()?.clone().into(), size)
        } else {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "The size limit of an array was exceeded.".into(),
                None,
                span,
            ));
        }
    }

    Ok(ThrushStatement::FixedArray {
        items,
        kind: array_type,
        span,
    })
}
