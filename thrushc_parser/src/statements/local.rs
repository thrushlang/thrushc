use thrushc_ast::{Ast, metadata::LocalMetadata, traits::AstGetType};
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_mir::atomicord::ThrushAtomicOrdering;
use thrushc_modificators::{Modificators, traits::ModificatorsExtensions};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::{Type, traits::InfererTypeExtensions};

use crate::{ParserContext, attributes, builder, expressions, typegen};

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
        builder::build_stmt_modificator(ctx, &[TokenType::Identifier])?;
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrushAtomicOrdering> = modificators.get_atomic_ordering();

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

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile, atomic_ord);

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
    }
}
