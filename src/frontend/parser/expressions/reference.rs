use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::frontend::types::ast::metadata::local::LocalMetadata;
use crate::frontend::types::ast::metadata::reference::{ReferenceMetadata, ReferenceType};
use crate::frontend::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontend::types::parser::stmts::traits::{
    FoundSymbolEither, FoundSymbolExtension, TokenExtensions,
};
use crate::frontend::types::parser::symbols::traits::{
    ConstantSymbolExtensions, FunctionParameterSymbolExtensions, LLISymbolExtensions,
    LocalSymbolExtensions, StaticSymbolExtensions,
};
use crate::frontend::types::parser::symbols::types::{
    ConstantSymbol, FoundSymbolId, Function, LLISymbol, LocalSymbol, ParameterSymbol, StaticSymbol,
};
use crate::frontend::typesystem::modificators::{
    FunctionReferenceTypeModificator, GCCFunctionReferenceTypeModificator,
    LLVMFunctionReferenceTypeModificator,
};
use crate::frontend::typesystem::types::Type;

pub fn build_reference<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_function() {
        let function_id: &str = symbol.expected_function(span)?;

        let function: Function = ctx.get_symbols().get_function_by_id(span, function_id)?;

        let function_type: Type = function.0;
        let function_parameter_types: Vec<Type> = function.1.0;
        let has_ignore_attr: bool = function.2;

        return Ok(Ast::Reference {
            name,
            kind: Type::Fn(
                function_parameter_types,
                function_type.into(),
                FunctionReferenceTypeModificator::new(
                    LLVMFunctionReferenceTypeModificator::new(has_ignore_attr),
                    GCCFunctionReferenceTypeModificator::new(),
                ),
            ),
            span,
            metadata: ReferenceMetadata::new(true, false, ReferenceType::default()),
        });
    }

    if symbol.is_static() {
        let static_var: (&str, usize) = symbol.expected_static(span)?;

        let static_id: &str = static_var.0;
        let scope_idx: usize = static_var.1;

        let static_var: StaticSymbol = ctx
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

        let constant: ConstantSymbol = ctx
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

        let parameter: ParameterSymbol =
            ctx.get_symbols().get_parameter_by_id(parameter_id, span)?;

        let metadata: FunctionParameterMetadata = parameter.get_metadata();
        let is_mutable: bool = metadata.is_mutable();

        let parameter_type: Type = parameter.get_type();

        let is_allocated: bool = parameter_type.is_ptr_type() || parameter_type.is_address_type();

        return Ok(Ast::Reference {
            name,
            kind: parameter_type,
            span,
            metadata: ReferenceMetadata::new(is_allocated, is_mutable, ReferenceType::default()),
        });
    }

    if symbol.is_lli() {
        let lli: (&str, usize) = symbol.expected_lli(span)?;

        let lli_id: &str = lli.0;
        let scope_idx: usize = lli.1;

        let parameter: &LLISymbol = ctx.get_symbols().get_lli_by_id(lli_id, scope_idx, span)?;

        let lli_type: Type = parameter.get_type();

        let is_allocated: bool = lli_type.is_ptr_type() || lli_type.is_address_type();

        return Ok(Ast::Reference {
            name,
            kind: lli_type,
            span,
            metadata: ReferenceMetadata::new(is_allocated, false, ReferenceType::default()),
        });
    }

    if symbol.is_local() {
        let local_position: (&str, usize) = symbol.expected_local(span)?;
        let local_id: &str = local_position.0;
        let scope_idx: usize = local_position.1;

        let local: &LocalSymbol = ctx
            .get_symbols()
            .get_local_by_id(local_id, scope_idx, span)?;

        let metadata: LocalMetadata = local.get_metadata();
        let is_mutable: bool = metadata.is_mutable();

        let local_type: Type = local.get_type();

        let reference: Ast = Ast::Reference {
            name,
            kind: local_type.clone(),
            span,
            metadata: ReferenceMetadata::new(true, is_mutable, ReferenceType::default()),
        };

        if ctx.match_token(TokenType::PlusPlus)? || ctx.match_token(TokenType::MinusMinus)? {
            let operator_tk: &Token = ctx.previous();
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

        return Ok(reference);
    }

    Err(ThrushCompilerIssue::Error(
        "Unknown reference".into(),
        "It is not a valid reference.".into(),
        None,
        span,
    ))
}
