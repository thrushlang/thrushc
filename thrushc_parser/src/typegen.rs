use thrushc_ast::{
    Ast,
    types::StructFields,
    traits::{AstGetType, AstStandardExtensions},
};
use thrushc_attributes::{ThrushAttributes, traits::ThrushAttributesExtensions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{
    Token,
    tokentype::TokenType,
    traits::{TokenExtensions, TokenTypeExtensions, TokenTypeTypeTransform},
};
use thrushc_typesystem::{
    Type,
    modificators::{
        FunctionReferenceTypeModificator, GCCFunctionReferenceTypeModificator,
        LLVMFunctionReferenceTypeModificator,
    },
    traits::{TypeCodeLocation, TypeIsExtensions},
};

use crate::{
    ParserContext, attributes,
    entities::{
        ConstantSymbol, CustomTypeSymbol, FoundSymbolId, LocalSymbol, ParameterSymbol,
        StaticSymbol, Struct,
    },
    expressions,
    traits::{
        FoundSymbolEitherExtensions, FoundSymbolExtensions, StructFieldsExtensions,
        StructSymbolExtensions,
    },
};

pub fn build_type(ctx: &mut ParserContext<'_>, parse_expr: bool) -> Result<Type, CompilationIssue> {
    match ctx.peek().kind {
        tk_kind if tk_kind.is_type() => {
            let tk: &Token = ctx.advance()?;
            let span: Span = tk.get_span();

            match tk_kind {
                _ if tk_kind.is_array() => self::build_array_type(ctx, span),
                _ if tk_kind.is_const() => self::build_const_type(ctx, span),
                _ if tk_kind.is_fn_ref() => self::build_fn_ref_type(ctx, span),
                _ => match tk_kind.as_type(span)? {
                    ty if ty.is_ptr_type() && ctx.check(TokenType::LBracket) => {
                        self::build_recursive_type(ctx, Type::Ptr(None, span), span)
                    }
                    ty => Ok(ty),
                },
            }
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = ctx.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;

            match object {
                _ if object.is_structure() => {
                    let (id, scope_idx) = object.expected_struct(span)?;
                    let structure: Struct =
                        ctx.get_symbols().get_struct_by_id(id, scope_idx, span)?;
                    let fields: StructFields = structure.get_fields();

                    Ok(fields.get_type())
                }
                _ if object.is_custom_type() => {
                    let (id, scope_idx) = object.expected_custom_type(span)?;
                    let custom: CustomTypeSymbol = ctx
                        .get_symbols()
                        .get_custom_type_by_id(id, scope_idx, span)?;

                    Ok(custom.0)
                }
                _ if object.is_parameter() => {
                    let parameter_id: &str = object.expected_parameter(span)?;
                    let parameter: ParameterSymbol =
                        ctx.get_symbols().get_parameter_by_id(parameter_id, span)?;

                    Ok(parameter.0)
                }
                _ if object.is_local() => {
                    let (id, scope_idx) = object.expected_local(span)?;
                    let local: LocalSymbol = ctx
                        .get_symbols()
                        .get_local_by_id(id, scope_idx, span)?
                        .clone();

                    Ok(local.0)
                }
                _ if object.is_static() => {
                    let (id, scope_idx) = object.expected_static(span)?;
                    let staticvar: StaticSymbol =
                        ctx.get_symbols().get_static_by_id(id, scope_idx, span)?;

                    Ok(staticvar.0)
                }
                _ if object.is_constant() => {
                    let (id, scope_idx) = object.expected_constant(span)?;
                    let constant: ConstantSymbol =
                        ctx.get_symbols().get_const_by_id(id, scope_idx, span)?;

                    Ok(constant.0)
                }
                _ => Err(CompilationIssue::Error(
                    CompilationIssueCode::E0001,
                    format!("Not found type '{}'.", name),
                    None,
                    span,
                )),
            }
        }

        _ if parse_expr => expressions::build_expr(ctx)?.get_value_type().cloned(),

        what_heck => Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            format!("Expected type, not '{}'", what_heck),
            None,
            ctx.previous().span,
        )),
    }
}

fn build_fn_ref_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, CompilationIssue> {
    ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0001,
        "Expected '['.".into(),
    )?;

    let mut parameter_types: Vec<Type> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let param_type: Type = self::build_type(ctx, false)?;

        if param_type.is_void_type() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0019,
                "Void type isn't a value.".into(),
                None,
                param_type.get_span(),
            ));
        }

        parameter_types.push(param_type);

        if ctx.check(TokenType::RBracket) {
            break;
        }

        ctx.consume(
            TokenType::Comma,
            CompilationIssueCode::E0001,
            "Expected ','.".into(),
        )?;
    }

    ctx.consume(
        TokenType::RBracket,
        CompilationIssueCode::E0001,
        "Expected ']'.".into(),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Arrow])?;
    let has_ignore: bool = attributes.has_ignore_attribute();

    ctx.consume(
        TokenType::Arrow,
        CompilationIssueCode::E0001,
        "Expected '->'.".into(),
    )?;

    let return_type: Type = self::build_type(ctx, false)?;

    Ok(Type::Fn(
        parameter_types,
        return_type.into(),
        FunctionReferenceTypeModificator::new(
            LLVMFunctionReferenceTypeModificator::new(has_ignore),
            GCCFunctionReferenceTypeModificator::default(),
        ),
        span,
    ))
}

fn build_const_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, CompilationIssue> {
    let inner_type: Type = self::build_type(ctx, false)?;

    if inner_type.is_void_type() {
        ctx.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            "Void type isn't a value.".into(),
            None,
            inner_type.get_span(),
        ));
    }

    Ok(Type::Const(inner_type.into(), span))
}

fn build_array_type(ctx: &mut ParserContext<'_>, span: Span) -> Result<Type, CompilationIssue> {
    ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0019,
        "Expected '['.".into(),
    )?;

    let array_type: Type = self::build_type(ctx, false)?;

    if array_type.is_void_type() {
        ctx.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            "Void type isn't a value.".into(),
            None,
            array_type.get_span(),
        ));
    }

    if ctx.check(TokenType::SemiColon) {
        ctx.consume(
            TokenType::SemiColon,
            CompilationIssueCode::E0001,
            "Expected ';'.".into(),
        )?;

        let size: Ast = expressions::build_expr(ctx)?;
        let size_type: &Type = size.get_value_type()?;

        if !size.is_integer() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected literal integer value as a size indicator.".into(),
                None,
                span,
            ));
        }

        if !size_type.is_unsigned_integer_type() || !size_type.is_lesseq_unsigned32bit_integer() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected unsigned integer value less than or equal to 32 bits.".into(),
                None,
                span,
            ));
        }

        let size: u64 = if let Ast::Integer { value, .. } = size {
            value
        } else {
            0
        };

        let array_size: Result<u32, std::num::TryFromIntError> = u32::try_from(size);

        if array_size.is_err() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected any unsigned 32 bits integer value.".into(),
                None,
                span,
            ));
        }

        ctx.consume(
            TokenType::RBracket,
            CompilationIssueCode::E0001,
            "Expected ']'.".into(),
        )?;

        return Ok(Type::FixedArray(
            array_type.into(),
            array_size.unwrap_or_default(),
            span,
        ));
    }

    ctx.consume(
        TokenType::RBracket,
        CompilationIssueCode::E0001,
        "Expected ']'.".into(),
    )?;

    Ok(Type::Array(array_type.into(), span))
}

fn build_recursive_type(
    ctx: &mut ParserContext<'_>,
    mut before_type: Type,
    span: Span,
) -> Result<Type, CompilationIssue> {
    ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0001,
        "Expected '['.".into(),
    )?;

    if let Type::Ptr(..) = &mut before_type {
        let mut inner_type: Type = self::build_type(ctx, false)?;

        if inner_type.is_void_type() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0019,
                "Void type isn't a value.".into(),
                None,
                inner_type.get_span(),
            ));
        }

        while ctx.check(TokenType::LBracket) {
            inner_type = self::build_recursive_type(ctx, inner_type, span)?;

            if inner_type.is_void_type() {
                ctx.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "Void type isn't a value.".into(),
                    None,
                    inner_type.get_span(),
                ));
            }
        }

        ctx.consume(
            TokenType::RBracket,
            CompilationIssueCode::E0001,
            "Expected ']'.".into(),
        )?;

        Ok(Type::Ptr(Some(inner_type.into()), span))
    } else {
        Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            format!("Expected pointer type, not '{}'", before_type),
            None,
            ctx.previous().span,
        ))
    }
}
