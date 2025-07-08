use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, typegen},
        types::{
            ast::Ast,
            parser::stmts::{
                traits::TokenExtensions,
                types::{CustomTypeFields, ThrushAttributes},
            },
        },
        typesystem::types::Type,
    },
};

pub fn build_custom_type<'parser>(
    parser_context: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    if !parser_context.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Types are only defined globally."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    parser_context.consume(
        TokenType::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    let name: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected type name."),
    )?;

    let custom_type_name: &str = name.get_lexeme();

    let span: Span = name.get_span();

    parser_context.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_context, &[TokenType::LBrace])?;

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut custom_type_fields: CustomTypeFields = Vec::with_capacity(10);

    while !parser_context.check(TokenType::RBrace) {
        let kind: Type = typegen::build_type(parser_context)?;
        custom_type_fields.push(kind);
    }

    parser_context.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_context.get_mut_symbols().new_custom_type(
            custom_type_name,
            (custom_type_fields, attributes),
            span,
        ) {
            parser_context.add_error(error);
        }
    }

    Ok(Ast::Null { span })
}
