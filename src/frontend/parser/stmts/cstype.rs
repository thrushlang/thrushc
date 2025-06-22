use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, typegen},
        types::{
            lexer::ThrushType,
            parser::stmts::{
                stmt::ThrushStatement,
                traits::TokenExtensions,
                types::{CustomTypeFields, ThrushAttributes},
            },
        },
    },
};

pub fn build_custom_type<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let type_tk: &Token = parser_ctx.consume(
        TokenType::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Types are only defined globally."),
            None,
            type_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected type name."),
    )?;

    let custom_type_name: &str = name.get_lexeme();

    let span: Span = name.get_span();

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let custom_typse_attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut custom_type_fields: CustomTypeFields = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenType::RBrace {
        let kind: ThrushType = typegen::build_type(parser_ctx)?;
        custom_type_fields.push(kind);
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_custom_type(
            custom_type_name,
            (custom_type_fields, custom_type_attributes),
            span,
        ) {
            parser_ctx.add_error(error);
        }
    }

    Ok(ThrushStatement::Null { span })
}
