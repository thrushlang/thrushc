use crate::{
    frontend::lexer::{span::Span, token::Token},
    middle::types::frontend::{
        lexer::{tokenkind::TokenKind, types::ThrushType},
        parser::{
            stmts::{
                traits::{
                    CustomTypeFieldsExtensions, FoundSymbolEither, FoundSymbolExtension,
                    StructExtensions, StructFieldsExtensions,
                },
                types::{CustomTypeFields, StructFields},
            },
            symbols::types::{CustomTypeSymbol, FoundSymbolId, Struct},
        },
    },
    standard::error::ThrushCompilerIssue,
};

use super::ParserContext;

pub fn build_type(parser_ctx: &mut ParserContext<'_>) -> Result<ThrushType, ThrushCompilerIssue> {
    let builded_type: Result<ThrushType, ThrushCompilerIssue> = match parser_ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = parser_ctx.advance()?;
            let span: Span = tk.span;

            if tk_kind.is_mut()
                && !parser_ctx.get_type_ctx().get_position().is_parameter()
                && !parser_ctx.get_type_ctx().get_position().is_bind_parameter()
            {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Mutable types is only allowed in function and bind parameters."),
                    None,
                    span,
                ));
            }

            if tk_kind.is_me()
                && !parser_ctx
                    .get_type_ctx()
                    .get_position()
                    .is_structure_field()
            {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Self types 'Me' is only allowed in structure fields."),
                    None,
                    span,
                ));
            }

            if tk_kind.is_me() {
                return Ok(ThrushType::Me(None));
            }

            if tk_kind.is_mut() {
                return Ok(ThrushType::Mut(build_type(parser_ctx)?.into()));
            }

            match tk_kind.as_type() {
                ty if ty.is_integer_type() => Ok(ty),
                ty if ty.is_float_type() => Ok(ty),
                ty if ty.is_bool_type() => Ok(ty),
                ty if ty.is_ptr_type() && parser_ctx.check(TokenKind::LBracket) => {
                    Ok(build_recursive_type(parser_ctx, ThrushType::Ptr(None))?)
                }
                ty if ty.is_ptr_type() => Ok(ty),
                ty if ty.is_void_type() => Ok(ty),
                ty if ty.is_str_type() => Ok(ty),

                what_heck => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    format!(
                        "The type '{}' cannot be a valid value in runtime.",
                        what_heck
                    ),
                    None,
                    span,
                )),
            }
        }

        TokenKind::Identifier => {
            let identifier_tk: &Token = parser_ctx.advance()?;

            let name: &str = identifier_tk.lexeme;
            let span: Span = identifier_tk.span;

            let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

            if object.is_structure() {
                let struct_id: &str = object.expected_struct(span)?;

                let structure: Struct =
                    parser_ctx.get_symbols().get_struct_by_id(struct_id, span)?;

                let fields: StructFields = structure.get_fields();

                Ok(fields.get_type())
            } else if object.is_custom_type() {
                let custom_id: &str = object.expected_custom_type(span)?;

                let custom: CustomTypeSymbol = parser_ctx
                    .get_symbols()
                    .get_custom_type_by_id(custom_id, span)?;

                let custom_type_fields: CustomTypeFields = custom.0;

                Ok(custom_type_fields.get_type())
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    format!("Not found type '{}'.", name),
                    None,
                    span,
                ));
            }
        }

        what_heck => Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Expected type, not '{}'", what_heck),
            None,
            parser_ctx.previous().span,
        )),
    };

    builded_type
}

fn build_recursive_type(
    parser_ctx: &mut ParserContext<'_>,
    mut before_type: ThrushType,
) -> Result<ThrushType, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenKind::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    if let ThrushType::Ptr(_) = &mut before_type {
        let mut inner_type: ThrushType = build_type(parser_ctx)?;

        while parser_ctx.check(TokenKind::LBracket) {
            inner_type = build_recursive_type(parser_ctx, inner_type)?;
        }

        parser_ctx.consume(
            TokenKind::RBracket,
            String::from("Syntax error"),
            String::from("Expected ']'."),
        )?;

        return Ok(ThrushType::Ptr(Some(inner_type.into())));
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        format!("Expected pointer type, not '{}'", before_type),
        None,
        parser_ctx.previous().span,
    ))
}
