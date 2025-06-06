use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        types::{
            lexer::ThrushType,
            parser::stmts::{
                traits::{
                    CustomTypeFieldsExtensions, FoundSymbolEither, FoundSymbolExtension,
                    StructExtensions, StructFieldsExtensions, TokenExtensions,
                },
                types::{CustomTypeFields, StructFields},
            },
            symbols::types::{CustomTypeSymbol, Struct},
        },
    },
};

use super::ParserContext;

pub fn build_type(parser_ctx: &mut ParserContext<'_>) -> Result<ThrushType, ThrushCompilerIssue> {
    let builded_type: Result<ThrushType, ThrushCompilerIssue> = match parser_ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = parser_ctx.advance()?;
            let span: Span = tk.span;

            if tk_kind.is_mut() {
                let inner_type: ThrushType = self::build_type(parser_ctx)?;

                if inner_type.is_mut_type() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Nested mutable type 'mut mut T' aren't allowed as type.".into(),
                        None,
                        span,
                    ));
                }

                if inner_type.is_ptr_type() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Mutable pointer 'mut ptr<T>' aren't allowed as type.".into(),
                        None,
                        span,
                    ));
                }

                return Ok(ThrushType::Mut(inner_type.into()));
            }

            match tk_kind.as_type(span)? {
                ty if ty.is_integer_type() => Ok(ty),
                ty if ty.is_float_type() => Ok(ty),
                ty if ty.is_bool_type() => Ok(ty),
                ty if ty.is_address_type() => Ok(ty),
                ty if ty.is_ptr_type() && parser_ctx.check(TokenType::LBracket) => Ok(
                    self::build_recursive_type(parser_ctx, ThrushType::Ptr(None))?,
                ),
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

        TokenType::Identifier => {
            let identifier_tk: &Token = parser_ctx.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            if let Ok(object) = parser_ctx.get_symbols().get_symbols_id(name, span) {
                if object.is_structure() {
                    let struct_id: &str = object.expected_struct(span)?;

                    let structure: Struct =
                        parser_ctx.get_symbols().get_struct_by_id(struct_id, span)?;

                    let fields: StructFields = structure.get_fields();

                    return Ok(fields.get_type());
                } else if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(span)?;

                    let custom: CustomTypeSymbol = parser_ctx
                        .get_symbols()
                        .get_custom_type_by_id(custom_id, span)?;

                    let custom_type_fields: CustomTypeFields = custom.0;

                    return Ok(custom_type_fields.get_type());
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        format!("Not found type '{}'.", name),
                        None,
                        span,
                    ));
                }
            }

            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                format!("Expected type, not '{}'", name),
                None,
                parser_ctx.previous().span,
            ));
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
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    if let ThrushType::Ptr(_) = &mut before_type {
        let mut inner_type: ThrushType = self::build_type(parser_ctx)?;

        while parser_ctx.check(TokenType::LBracket) {
            inner_type = self::build_recursive_type(parser_ctx, inner_type)?;
        }

        parser_ctx.consume(
            TokenType::RBracket,
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
