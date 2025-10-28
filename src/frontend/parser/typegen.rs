use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::attributes;
use crate::frontend::parser::expr;
use crate::frontend::types::ast::Ast;

use crate::frontend::types::parser::stmts::traits::{
    FoundSymbolEither, FoundSymbolExtension, StructExtensions, StructFieldsExtensions,
    ThrushAttributesExtensions, TokenExtensions,
};
use crate::frontend::types::parser::stmts::types::{StructFields, ThrushAttributes};
use crate::frontend::types::parser::symbols::types::{CustomTypeSymbol, Struct};
use crate::frontend::typesystem::modificators::{
    FunctionReferenceTypeModificator, GCCFunctionReferenceTypeModificator,
    LLVMFunctionReferenceTypeModificator,
};
use crate::frontend::typesystem::types::Type;

use super::ParserContext;

pub fn build_type(ctx: &mut ParserContext<'_>) -> Result<Type, ThrushCompilerIssue> {
    match ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = ctx.advance()?;
            let span: Span = tk.span;

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
                ty if ty.is_ptr_type() && ctx.check(TokenType::LBracket) => {
                    self::build_recursive_type(ctx, Type::Ptr(None))
                }
                ty => Ok(ty),
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

                    Ok(fields.get_type())
                } else if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(span)?;
                    let custom: CustomTypeSymbol =
                        ctx.get_symbols().get_custom_type_by_id(custom_id, span)?;

                    Ok(custom.0)
                } else {
                    Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        format!("Not found type '{}'.", name),
                        None,
                        span,
                    ))
                }
            } else {
                Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    format!("Expected type, not '{}'", name),
                    None,
                    ctx.previous().span,
                ))
            }
        }

        what_heck => Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!("Expected type, not '{}'", what_heck),
            None,
            ctx.previous().span,
        )),
    }
}

fn build_fn_ref_type(ctx: &mut ParserContext<'_>) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        "Syntax error".into(),
        "Expected '['.".into(),
    )?;

    let mut parameter_types: Vec<Type> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        parameter_types.push(build_type(ctx)?);

        if ctx.check(TokenType::RBracket) {
            break;
        }

        ctx.consume(
            TokenType::Comma,
            "Syntax error".into(),
            "Expected ','.".into(),
        )?;
    }

    ctx.consume(
        TokenType::RBracket,
        "Syntax error".into(),
        "Expected ']'.".into(),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Arrow])?;
    let has_ignore: bool = attributes.has_ignore_attribute();

    ctx.consume(
        TokenType::Arrow,
        "Syntax error".into(),
        "Expected '->'.".into(),
    )?;

    let return_type: Type = build_type(ctx)?;

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
    Ok(Type::Const(build_type(ctx)?.into()))
}

fn build_array_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        "Syntax error".into(),
        "Expected '['.".into(),
    )?;

    let array_type: Type = build_type(ctx)?;

    if ctx.check(TokenType::SemiColon) {
        ctx.consume(
            TokenType::SemiColon,
            "Syntax error".into(),
            "Expected ';'.".into(),
        )?;

        let size: Ast = expr::build_expr(ctx)?;

        if !size.is_integer() {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected integer value.".into(),
                None,
                span,
            ));
        }

        if !size.is_unsigned_integer()? || !size.is_lessu32bit_integer()? {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected any unsigned integer value less than or equal to 32 bits.".into(),
                None,
                span,
            ));
        }

        let raw_array_size: u64 = size.get_integer_value()?;

        if let Ok(array_size) = u32::try_from(raw_array_size) {
            ctx.consume(
                TokenType::RBracket,
                "Syntax error".into(),
                "Expected ']'.".into(),
            )?;

            return Ok(Type::FixedArray(array_type.into(), array_size));
        }

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Expected any unsigned 32 bits integer value.".into(),
            None,
            span,
        ));
    }

    ctx.consume(
        TokenType::RBracket,
        "Syntax error".into(),
        "Expected ']'.".into(),
    )?;
    Ok(Type::Array(array_type.into()))
}

fn build_recursive_type(
    ctx: &mut ParserContext<'_>,
    mut before_type: Type,
) -> Result<Type, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBracket,
        "Syntax error".into(),
        "Expected '['.".into(),
    )?;

    if let Type::Ptr(_) = &mut before_type {
        let mut inner_type: Type = build_type(ctx)?;

        while ctx.check(TokenType::LBracket) {
            inner_type = build_recursive_type(ctx, inner_type)?;
        }

        ctx.consume(
            TokenType::RBracket,
            "Syntax error".into(),
            "Expected ']'.".into(),
        )?;

        Ok(Type::Ptr(Some(inner_type.into())))
    } else {
        Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!("Expected pointer type, not '{}'", before_type),
            None,
            ctx.previous().span,
        ))
    }
}
