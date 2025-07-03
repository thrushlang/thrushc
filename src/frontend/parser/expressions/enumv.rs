use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::Ast,
            lexer::Type,
            parser::{
                stmts::{
                    traits::{
                        EnumExtensions, EnumFieldsExtensions, FoundSymbolEither, TokenExtensions,
                    },
                    types::{EnumField, EnumFields},
                },
                symbols::types::FoundSymbolId,
            },
        },
    },
};

pub fn build_enum_value<'parser>(
    parser_context: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;
    let enum_id: &str = object.expected_enum(span)?;

    let union: EnumFields = parser_context
        .get_symbols()
        .get_enum_by_id(enum_id, span)?
        .get_fields();

    let field_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let field_name: &str = field_tk.get_lexeme();

    if !union.contain_field(field_name) {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Not found '{}' field in '{}' enum.", name, field_name),
            None,
            span,
        ));
    }

    let field: EnumField = union.get_field(field_name);
    let field_value: Ast = field.1;
    let field_type: Type = field_value.get_value_type()?.clone();

    let canonical_name: String = format!("{}.{}", name, field_name);

    Ok(Ast::EnumValue {
        name: canonical_name,
        value: field_value.into(),
        kind: field_type,
        span,
    })
}
