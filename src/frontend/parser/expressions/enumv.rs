use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::Ast,
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
        typesystem::types::Type,
    },
};

pub fn build_enum_value<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;
    let enum_id: &str = object.expected_enum(span)?;

    let union: EnumFields = ctx
        .get_symbols()
        .get_enum_by_id(enum_id, span)?
        .get_fields();

    let field_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected enum name.".into(),
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

    let field_type: Type = field.1;
    let field_value: Ast = field.2;

    let canonical_name: String = format!("{}.{}", name, field_name);

    Ok(Ast::EnumValue {
        name: canonical_name,
        value: field_value.into(),
        kind: field_type,
        span,
    })
}
