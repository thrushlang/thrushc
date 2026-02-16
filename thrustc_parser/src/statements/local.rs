use thrustc_ast::{Ast, metadata::LocalMetadata, traits::AstGetType};
use thrustc_attributes::ThrustAttributes;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_mir::atomicord::ThrustAtomicOrdering;
use thrustc_modificators::{Modificators, traits::ModificatorsExtensions};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::{Type, traits::InfererTypeExtensions};

use crate::{ParserContext, attributes, expressions, modificators, typegen};

pub fn build_local<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Local,
        CompilationIssueCode::E0001,
        "Expected 'local' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

    let modificators: Modificators =
        modificators::build_stmt_modificator(ctx, &[TokenType::Identifier])?;
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrustAtomicOrdering> = modificators.get_atomic_ordering();

    let local_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();
    let span: Span = local_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        String::from("Expected ':'."),
    )?;

    let mut local_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile, atomic_ord);

        if !ctx.is_main_scope() {
            ctx.get_mut_symbols()
                .new_local(name, (local_type.clone(), metadata, span), span)?;

            let local: Ast = Ast::Local {
                name,
                ascii_name,
                kind: local_type,
                value: None,
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(local)
        } else {
            Ok(Ast::invalid_ast(span))
        }
    } else {
        let metadata: LocalMetadata =
            LocalMetadata::new(false, is_mutable, is_volatile, atomic_ord);

        ctx.consume(
            TokenType::Eq,
            CompilationIssueCode::E0001,
            String::from("Expected '='."),
        )?;

        ctx.get_mut_type_ctx().add_infered_type(local_type.clone());

        let value: Ast = expressions::build_expression(ctx)?;
        let value_type: &Type = value.get_value_type()?;

        ctx.get_mut_type_ctx().pop_infered_type();

        local_type.inferer_inner_type_from_type(value_type);

        if !ctx.is_main_scope() {
            ctx.get_mut_symbols()
                .new_local(name, (local_type.clone(), metadata, span), span)?;

            let local: Ast = Ast::Local {
                name,
                ascii_name,
                kind: local_type,
                value: Some(value.into()),
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(local)
        } else {
            Ok(Ast::invalid_ast(span))
        }
    }
}

pub fn build_local_as_not_inserted_yet<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Local,
        CompilationIssueCode::E0001,
        "Expected 'local' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

    let modificators: Modificators =
        modificators::build_stmt_modificator(ctx, &[TokenType::Identifier])?;
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrustAtomicOrdering> = modificators.get_atomic_ordering();

    let local_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();
    let span: Span = local_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        String::from("Expected ':'."),
    )?;

    let mut local_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile, atomic_ord);

        if !ctx.is_main_scope() {
            let local: Ast = Ast::Local {
                name,
                ascii_name,
                kind: local_type,
                value: None,
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(local)
        } else {
            Ok(Ast::invalid_ast(span))
        }
    } else {
        let metadata: LocalMetadata =
            LocalMetadata::new(false, is_mutable, is_volatile, atomic_ord);

        ctx.consume(
            TokenType::Eq,
            CompilationIssueCode::E0001,
            String::from("Expected '='."),
        )?;

        ctx.get_mut_type_ctx().add_infered_type(local_type.clone());

        let value: Ast = expressions::build_expression(ctx)?;
        let value_type: &Type = value.get_value_type()?;

        ctx.get_mut_type_ctx().pop_infered_type();

        local_type.inferer_inner_type_from_type(value_type);

        if !ctx.is_main_scope() {
            let local: Ast = Ast::Local {
                name,
                ascii_name,
                kind: local_type,
                value: Some(value.into()),
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(local)
        } else {
            Ok(Ast::invalid_ast(span))
        }
    }
}
