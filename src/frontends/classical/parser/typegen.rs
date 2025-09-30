use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{attributes, expr},
        types::{
            ast::Ast,
            parser::{
                stmts::{
                    traits::{
                        FoundSymbolEither, FoundSymbolExtension, StructExtensions,
                        StructFieldsExtensions, ThrushAttributesExtensions, TokenExtensions,
                    },
                    types::{StructFields, ThrushAttributes},
                },
                symbols::types::{CustomTypeSymbol, Struct},
            },
        },
        typesystem::{
            modificators::{
                FunctionReferenceTypeModificator, GCCFunctionReferenceTypeModificator,
                LLVMFunctionReferenceTypeModificator,
            },
            types::Type,
        },
    },
};

use super::ParserContext;

pub fn build_type(ctx: &mut ParserContext<'_>) -> Result<Type, ThrushCompilerIssue> {
    let builded_type: Result<Type, ThrushCompilerIssue> = match ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = ctx.advance()?;
            let span: Span = tk.span;

            if tk_kind.is_mut() {
                return self::build_mut_type(ctx, span);
            }

            if tk_kind.is_array() {
                return self::build_array_type(ctx, span);
            }

            if tk_kind.is_const() {
                return self::build_const_type(ctx);
            }

            if tk_kind.is_fn_ref() {
                return self::build_fn_ref_type(ctx);
            }

            match tk_kind.as_type(span)? {
                ty if ty.is_integer_type() => Ok(ty),
                ty if ty.is_float_type() => Ok(ty),
                ty if ty.is_bool_type() => Ok(ty),
                ty if ty.is_address_type() => Ok(ty),
                ty if ty.is_ptr_type() && ctx.check(TokenType::LBracket) => {
                    Ok(self::build_recursive_type(ctx, Type::Ptr(None))?)
                }
                ty if ty.is_ptr_type() => Ok(ty),
                ty if ty.is_void_type() => Ok(ty),

                what_heck => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    format!("Expected type, not '{}'", what_heck),
                    None,
                    span,
                )),
            }
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = ctx.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            if let Ok(object) = ctx.get_symbols().get_symbols_id(name, span) {
                if object.is_structure() {
                    let struct_id: &str = object.expected_struct(span)?;

                    let structure: Struct = ctx.get_symbols().get_struct_by_id(struct_id, span)?;

                    let fields: StructFields = structure.get_fields();

                    return Ok(fields.get_type());
                } else if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(span)?;

                    let custom: CustomTypeSymbol =
                        ctx.get_symbols().get_custom_type_by_id(custom_id, span)?;

                    let custom_type: Type = custom.0;

                    return Ok(custom_type);
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
                ctx.previous().span,
            ));
        }

        what_heck => Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Expected type, not '{}'", what_heck),
            None,
            ctx.previous().span,
        )),
    };

    builded_type
}

fn build_mut_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, ThrushCompilerIssue> {
    let inner_type: Type = self::build_type(ctx)?;

    if inner_type.is_mut_type() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "Nested mutable type 'mut mut T' ins't allowed.".into(),
            None,
            span,
        ));
    }

    if inner_type.is_ptr_type() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "Mutable pointer type 'mut ptr<T>', or 'mut ptr' isn't allowed.".into(),
            None,
            span,
        ));
    }

    Ok(Type::Mut(inner_type.into()))
}

fn build_fn_ref_type(ctx: &mut ParserContext<'_>) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let mut parameter_types: Vec<Type> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let parameter_type: Type = self::build_type(ctx)?;

        parameter_types.push(parameter_type);

        if ctx.check(TokenType::RBracket) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Arrow])?;

    let has_ignore: bool = attributes.has_ignore_attribute();

    ctx.consume(
        TokenType::Arrow,
        String::from("Syntax error"),
        String::from("Expected '->'."),
    )?;

    let return_type: Type = self::build_type(ctx)?;

    Ok(Type::Fn(
        parameter_types,
        return_type.into(),
        FunctionReferenceTypeModificator::new(
            LLVMFunctionReferenceTypeModificator::new(has_ignore),
            GCCFunctionReferenceTypeModificator::default(),
        ),
    ))
}

fn build_const_type(ctx: &mut ParserContext<'_>) -> Result<Type, ThrushCompilerIssue> {
    let inner_type: Type = self::build_type(ctx)?;

    Ok(Type::Const(inner_type.into()))
}

fn build_array_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let array_type: Type = self::build_type(ctx)?;

    if ctx.check(TokenType::SemiColon) {
        ctx.consume(
            TokenType::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        let size: Ast = expr::build_expr(ctx)?;

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
            ctx.consume(
                TokenType::RBracket,
                String::from("Syntax error"),
                String::from("Expected ']'."),
            )?;

            return Ok(Type::FixedArray(array_type.into(), array_size));
        }

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "Expected any unsigned 32 bits integer value.".into(),
            None,
            span,
        ));
    }

    ctx.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    Ok(Type::Array(array_type.into()))
}

fn build_recursive_type(
    ctx: &mut ParserContext<'_>,
    mut before_type: Type,
) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    if let Type::Ptr(_) = &mut before_type {
        let mut inner_type: Type = self::build_type(ctx)?;

        while ctx.check(TokenType::LBracket) {
            inner_type = self::build_recursive_type(ctx, inner_type)?;
        }

        ctx.consume(
            TokenType::RBracket,
            String::from("Syntax error"),
            String::from("Expected ']'."),
        )?;

        return Ok(Type::Ptr(Some(inner_type.into())));
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        format!("Expected pointer type, not '{}'", before_type),
        None,
        ctx.previous().span,
    ))
}
