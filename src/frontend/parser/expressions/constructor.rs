use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::parser::{
            stmts::{
                stmt::ThrushStatement,
                traits::{ConstructorExtensions, StructExtensions, TokenExtensions},
                types::Constructor,
            },
            symbols::types::Struct,
        },
    },
};

pub fn build_constructor<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let new_tk: &Token = parser_context.consume(
        TokenType::New,
        String::from("Syntax error"),
        String::from("Expected 'new' keyword."),
    )?;

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            new_tk.span,
        ));
    }

    let name: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure reference."),
    )?;

    let struct_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let struct_found: Struct = parser_context.get_symbols().get_struct(struct_name, span)?;
    let fields_required: usize = struct_found.get_fields().1.len();

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut arguments: Constructor = (struct_name, Vec::with_capacity(10));

    let mut amount: usize = 0;

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        if parser_context.match_token(TokenType::Identifier)? {
            let field_tk: &Token = parser_context.previous();
            let field_span: Span = field_tk.span;
            let field_name: &str = field_tk.get_lexeme();

            parser_context.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            if !struct_found.contains_field(field_name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected existing structure field name."),
                    None,
                    field_span,
                ));
            }

            if amount >= fields_required {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Too many fields in structure"),
                    format!("Expected '{}' fields, not '{}'.", fields_required, amount),
                    None,
                    span,
                ));
            }

            let expression: ThrushStatement = expression::build_expr(parser_context)?;

            if expression.is_constructor() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Constructor should be stored in a local variable."),
                    None,
                    field_span,
                ));
            }

            if let Some(target_type) = struct_found.get_field_type(field_name) {
                arguments
                    .1
                    .push((field_name, expression, target_type, amount as u32));
            }

            amount += 1;

            if parser_context.check(TokenType::RBrace) {
                break;
            }

            if parser_context.match_token(TokenType::Comma)? {
                if parser_context.check(TokenType::RBrace) {
                    break;
                }
            } else if parser_context.check_to(TokenType::Identifier, 0) {
                parser_context.consume(
                    TokenType::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    parser_context.previous().get_span(),
                ));
            }
        } else {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected field name."),
                None,
                span,
            ));
        }
    }

    let amount_fields: usize = arguments.1.len();

    if amount_fields != fields_required {
        return Err(ThrushCompilerIssue::Error(
            String::from("Missing fields in structure"),
            format!(
                "Expected '{}' arguments, but '{}' was gived.",
                fields_required, amount_fields
            ),
            None,
            span,
        ));
    }

    parser_context.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    Ok(ThrushStatement::Constructor {
        name: struct_name,
        arguments: arguments.clone(),
        kind: arguments.get_type(),
        span,
    })
}
