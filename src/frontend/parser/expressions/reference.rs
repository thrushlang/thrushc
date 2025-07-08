use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::ParserContext,
        types::{
            ast::{
                Ast,
                metadata::{
                    reference::{ReferenceMetadata, ReferenceType},
                    staticvar::StaticMetadata,
                },
            },
            parser::{
                stmts::traits::{FoundSymbolEither, FoundSymbolExtension, TokenExtensions},
                symbols::{
                    traits::{
                        ConstantSymbolExtensions, LLISymbolExtensions, LocalSymbolExtensions,
                        StaticSymbolExtensions,
                    },
                    types::{
                        ConstantSymbol, FoundSymbolId, LLISymbol, LocalSymbol, ParameterSymbol,
                        StaticSymbol,
                    },
                },
            },
        },
        typesystem::types::Type,
    },
};

pub fn build_reference<'parser>(
    parser_context: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_static() {
        let static_var: (&str, usize) = symbol.expected_static(span)?;

        let static_id: &str = static_var.0;
        let scope_idx: usize = static_var.1;

        let static_var: StaticSymbol = parser_context
            .get_symbols()
            .get_static_by_id(static_id, scope_idx, span)?;

        let static_type: Type = static_var.get_type();

        let metadata: StaticMetadata = static_var.get_metadata();

        let is_mutable: bool = metadata.is_mutable();

        return Ok(Ast::Reference {
            name,
            kind: static_type,
            span,
            metadata: ReferenceMetadata::new(true, is_mutable, ReferenceType::default()),
        });
    }

    if symbol.is_constant() {
        let constant: (&str, usize) = symbol.expected_constant(span)?;

        let const_id: &str = constant.0;
        let scope_idx: usize = constant.1;

        let constant: ConstantSymbol = parser_context
            .get_symbols()
            .get_const_by_id(const_id, scope_idx, span)?;

        let constant_type: Type = constant.get_type();

        return Ok(Ast::Reference {
            name,
            kind: constant_type,
            span,
            metadata: ReferenceMetadata::new(true, false, ReferenceType::Constant),
        });
    }

    if symbol.is_parameter() {
        let parameter_id: &str = symbol.expected_parameter(span)?;

        let parameter: ParameterSymbol = parser_context
            .get_symbols()
            .get_parameter_by_id(parameter_id, span)?;

        let parameter_type: Type = parameter.get_type();

        let is_mutable: bool = parameter.is_mutable();

        let is_allocated: bool = parameter_type.is_mut_type()
            || parameter_type.is_ptr_type()
            || parameter_type.is_address_type();

        return Ok(Ast::Reference {
            name,
            kind: parameter_type,
            span,
            metadata: ReferenceMetadata::new(is_allocated, is_mutable, ReferenceType::default()),
        });
    }

    if symbol.is_lli() {
        let lli_id: (&str, usize) = symbol.expected_lli(span)?;

        let lli_name: &str = lli_id.0;
        let scope_idx: usize = lli_id.1;

        let parameter: &LLISymbol = parser_context
            .get_symbols()
            .get_lli_by_id(lli_name, scope_idx, span)?;

        let lli_type: Type = parameter.get_type();

        let is_allocated: bool = lli_type.is_ptr_type() || lli_type.is_address_type();

        return Ok(Ast::Reference {
            name,
            kind: lli_type,
            span,
            metadata: ReferenceMetadata::new(is_allocated, false, ReferenceType::default()),
        });
    }

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &LocalSymbol =
        parser_context
            .get_symbols()
            .get_local_by_id(local_position.0, local_position.1, span)?;

    let is_mutable: bool = local.is_mutable();

    let local_type: Type = local.get_type();

    let reference: Ast = Ast::Reference {
        name,
        kind: local_type.clone(),
        span,
        metadata: ReferenceMetadata::new(true, is_mutable, ReferenceType::default()),
    };

    if parser_context.match_token(TokenType::PlusPlus)?
        | parser_context.match_token(TokenType::MinusMinus)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let unaryop: Ast = Ast::UnaryOp {
            operator,
            expression: reference.into(),
            kind: local_type,
            is_pre: false,
            span,
        };

        return Ok(unaryop);
    }

    Ok(reference)
}
