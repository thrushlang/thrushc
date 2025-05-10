use crate::{
    common::error::ThrushCompilerError,
    middle::{
        statement::{
            CustomType, CustomTypeFields, StructFields,
            traits::{
                CustomTypeFieldsExtensions, FoundSymbolEither, FoundSymbolExtension,
                StructExtensions, StructFieldsExtensions,
            },
        },
        symbols::types::{FoundSymbolId, Struct},
        types::{TokenKind, Type},
    },
};

use super::{
    lexer::{Span, Token},
    parser::ParserContext,
};

pub fn build_type(
    parser_ctx: &mut ParserContext<'_>,
    consume: Option<TokenKind>,
) -> Result<Type, ThrushCompilerError> {
    let builded_type: Result<Type, ThrushCompilerError> = match parser_ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = parser_ctx.advance()?;

            if tk_kind.is_mut()
                && !parser_ctx.get_type_ctx().get_position().is_parameter()
                && !parser_ctx.get_type_ctx().get_position().is_bind_parameter()
            {
                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Mutable types is only allowed in function and bind parameters."),
                    String::default(),
                    tk.span,
                ));
            }

            if tk_kind.is_mut() {
                return Ok(Type::Mut(build_type(parser_ctx, consume)?.into()));
            }

            match tk_kind.as_type() {
                ty if ty.is_integer_type() => Ok(ty),
                ty if ty.is_float_type() => Ok(ty),
                ty if ty.is_bool_type() => Ok(ty),
                ty if ty.is_ptr_type() && parser_ctx.check(TokenKind::LBracket) => {
                    Ok(build_recursive_type(parser_ctx, Type::Ptr(None))?)
                }
                ty if ty.is_ptr_type() => Ok(ty),
                ty if ty.is_void_type() => Ok(ty),
                ty if ty.is_str_type() => Ok(ty),

                what_heck => Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    format!(
                        "The type '{}' cannot be a value during the compile time.",
                        what_heck
                    ),
                    String::default(),
                    tk.span,
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

                let custom: CustomType = parser_ctx
                    .get_symbols()
                    .get_custom_type_by_id(custom_id, span)?;

                let custom_type_fields: CustomTypeFields = custom.0;

                Ok(custom_type_fields.get_type())
            } else {
                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    format!("Not found type '{}'.", name),
                    String::default(),
                    span,
                ));
            }
        }

        what_heck => Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            format!("Expected type, not '{}'", what_heck),
            String::default(),
            parser_ctx.previous().span,
        )),
    };

    if let Some(tk_kind) = consume {
        parser_ctx.consume(
            tk_kind,
            String::from("Syntax error"),
            format!("Expected '{}'.", tk_kind),
        )?;
    }

    builded_type
}

fn build_recursive_type(
    parser_ctx: &mut ParserContext<'_>,
    mut before_type: Type,
) -> Result<Type, ThrushCompilerError> {
    parser_ctx.consume(
        TokenKind::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    if let Type::Ptr(_) = &mut before_type {
        let mut inner_type: Type = build_type(parser_ctx, None)?;

        while parser_ctx.check(TokenKind::LBracket) {
            inner_type = build_recursive_type(parser_ctx, inner_type)?;
        }

        parser_ctx.consume(
            TokenKind::RBracket,
            String::from("Syntax error"),
            String::from("Expected ']'."),
        )?;

        return Ok(Type::Ptr(Some(inner_type.into())));
    }

    Err(ThrushCompilerError::Error(
        String::from("Syntax error"),
        format!("Expected pointer type, not '{}'", before_type),
        String::default(),
        parser_ctx.previous().span,
    ))
}
