use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::EnumExtensions;
use crate::front_end::types::parser::stmts::traits::EnumFieldsExtensions;
use crate::front_end::types::parser::stmts::traits::FoundSymbolEither;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::EnumField;
use crate::front_end::types::parser::stmts::types::EnumFields;
use crate::front_end::types::parser::symbols::types::FoundSymbolId;
use crate::front_end::typesystem::types::Type;

pub fn build_enum_value<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;
    let enum_id: (&str, usize) = object.expected_enum(span)?;
    let id: &str = enum_id.0;
    let scope_idx: usize = enum_id.1;

    let union: EnumFields = ctx
        .get_symbols()
        .get_enum_by_id(id, scope_idx, span)?
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
