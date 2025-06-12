use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::expression,
        types::{
            lexer::ThrushType,
            parser::stmts::{
                stmt::ThrushStatement,
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

pub fn build_type(
    parser_context: &mut ParserContext<'_>,
) -> Result<ThrushType, ThrushCompilerIssue> {
    let builded_type: Result<ThrushType, ThrushCompilerIssue> = match parser_context.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = parser_context.advance()?;
            let span: Span = tk.span;

            if tk_kind.is_mut() {
                let inner_type: ThrushType = self::build_type(parser_context)?;

                if inner_type.is_mut_type() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Nested mutable type 'mut mut T' ins't type.".into(),
                        None,
                        span,
                    ));
                }

                if inner_type.is_ptr_type() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Mutable pointer type 'mut ptr<T>', or 'mut ptr' isn't type.".into(),
                        None,
                        span,
                    ));
                }

                return Ok(ThrushType::Mut(inner_type.into()));
            }

            if tk_kind.is_array() {
                parser_context.consume(
                    TokenType::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                let array_type: ThrushType = self::build_type(parser_context)?;

                parser_context.consume(
                    TokenType::SemiColon,
                    String::from("Syntax error"),
                    String::from("Expected ';'."),
                )?;

                let size: ThrushStatement = expression::build_expr(parser_context)?;

                if !size.is_integer() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Expected integer value.".into(),
                        None,
                        span,
                    ));
                }

                if !size.is_unsigned_integer()? || !size.is_lessu32bit_integer()? {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Expected any unsigned integer value less than or equal to 32 bits.".into(),
                        None,
                        span,
                    ));
                }

                let raw_array_size: u64 = size.get_integer_value()?;

                if let Ok(array_size) = u32::try_from(raw_array_size) {
                    parser_context.consume(
                        TokenType::RBracket,
                        String::from("Syntax error"),
                        String::from("Expected ']'."),
                    )?;

                    return Ok(ThrushType::FixedArray(array_type.into(), array_size));
                }

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    "Expected any unsigned 32 bits integer value.".into(),
                    None,
                    span,
                ));
            }

            if tk_kind.is_str() {
                parser_context.consume(
                    TokenType::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                let size: ThrushStatement = expression::build_expr(parser_context)?;

                if !size.is_integer() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Expected integer value.".into(),
                        None,
                        span,
                    ));
                }

                if !size.is_unsigned_integer()? || !size.is_lessu32bit_integer()? {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        "Expected any unsigned integer value less than or equal to 32 bits.".into(),
                        None,
                        span,
                    ));
                }

                let raw_array_size: u64 = size.get_integer_value()?;

                if let Ok(array_size) = u32::try_from(raw_array_size) {
                    parser_context.consume(
                        TokenType::RBracket,
                        String::from("Syntax error"),
                        String::from("Expected ']'."),
                    )?;

                    return Ok(ThrushType::Str(
                        ThrushType::FixedArray(ThrushType::U8.into(), array_size).into(),
                    ));
                }

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    "Expected any unsigned 32 bits integer value.".into(),
                    None,
                    span,
                ));
            }

            match tk_kind.as_type(span)? {
                ty if ty.is_integer_type() => Ok(ty),
                ty if ty.is_float_type() => Ok(ty),
                ty if ty.is_bool_type() => Ok(ty),
                ty if ty.is_address_type() => Ok(ty),
                ty if ty.is_ptr_type() && parser_context.check(TokenType::LBracket) => Ok(
                    self::build_recursive_type(parser_context, ThrushType::Ptr(None))?,
                ),
                ty if ty.is_ptr_type() => Ok(ty),
                ty if ty.is_void_type() => Ok(ty),

                what_heck => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    format!("Type '{}' ins't value.", what_heck),
                    None,
                    span,
                )),
            }
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = parser_context.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            if let Ok(object) = parser_context.get_symbols().get_symbols_id(name, span) {
                if object.is_structure() {
                    let struct_id: &str = object.expected_struct(span)?;

                    let structure: Struct = parser_context
                        .get_symbols()
                        .get_struct_by_id(struct_id, span)?;

                    let fields: StructFields = structure.get_fields();

                    return Ok(fields.get_type());
                } else if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(span)?;

                    let custom: CustomTypeSymbol = parser_context
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
                parser_context.previous().span,
            ));
        }

        what_heck => Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Expected type, not '{}'", what_heck),
            None,
            parser_context.previous().span,
        )),
    };

    builded_type
}

fn build_recursive_type(
    parser_context: &mut ParserContext<'_>,
    mut before_type: ThrushType,
) -> Result<ThrushType, ThrushCompilerIssue> {
    parser_context.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    if let ThrushType::Ptr(_) = &mut before_type {
        let mut inner_type: ThrushType = self::build_type(parser_context)?;

        while parser_context.check(TokenType::LBracket) {
            inner_type = self::build_recursive_type(parser_context, inner_type)?;
        }

        parser_context.consume(
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
        parser_context.previous().span,
    ))
}
