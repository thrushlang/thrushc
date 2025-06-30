use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{
            ast::Ast,
            parser::{
                stmts::{
                    traits::{ConstructorExtensions, StructExtensions, TokenExtensions},
                    types::Constructor,
                },
                symbols::types::Struct,
            },
        },
    },
};

pub fn build_constructor<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_context.previous().span,
        ));
    }

    parser_context.consume(
        TokenType::New,
        "Syntax error".into(),
        "Expected 'new' keyword.".into(),
    )?;

    let struct_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected 'identifier' keyword.".into(),
    )?;

    let name: &str = struct_tk.get_lexeme();
    let span: Span = struct_tk.get_span();

    let struct_found: Struct = parser_context.get_symbols().get_struct(name, span)?;
    let fields_required: usize = struct_found.get_fields().1.len();

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut args: Constructor = Vec::with_capacity(10);

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

            let expression: Ast = expression::build_expr(parser_context)?;

            if let Some(target_type) = struct_found.get_field_type(field_name) {
                args.push((field_name, expression, target_type, amount as u32));
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

    let amount_fields: usize = args.len();

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

    Ok(Ast::Constructor {
        name,
        args: args.clone(),
        kind: args.get_type(name),
        span,
    })
}
