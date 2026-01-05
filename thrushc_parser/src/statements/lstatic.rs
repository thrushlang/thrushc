use thrushc_ast::{Ast, metadata::StaticMetadata};
use thrushc_attributes::{ThrushAttributes, traits::ThrushAttributesExtensions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_mir::{atomicord::ThrushAtomicOrdering, threadmode::ThrushThreadMode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, builder, expressions, typegen};

pub fn build_static<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Static,
        CompilationIssueCode::E0001,
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

    let thread_local: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atomic_ord: Option<ThrushAtomicOrdering> = builder::build_atomic_ord(ctx)?;
    let thread_mode: Option<ThrushThreadMode> = builder::build_thread_local_mode(ctx)?;

    let static_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected name.".into(),
    )?;

    let name: &str = static_tk.get_lexeme();
    let ascii_name: &str = static_tk.get_ascii_lexeme();

    let span: Span = static_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        "Expected ':'.".into(),
    )?;

    let static_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;
    let external: bool = attributes.has_extern_attribute();

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: StaticMetadata = StaticMetadata::new(
            true,
            is_mutable,
            true,
            thread_local,
            is_volatile,
            external,
            atomic_ord,
            thread_mode,
        );

        return Ok(Ast::Static {
            name,
            ascii_name,
            kind: static_type,
            value: None,
            attributes,
            metadata,
            span,
        });
    }

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        "Expected '='.".into(),
    )?;

    let value: Ast = expressions::build_expression(ctx)?;

    let metadata: StaticMetadata = StaticMetadata::new(
        true,
        is_mutable,
        false,
        thread_local,
        is_volatile,
        external,
        atomic_ord,
        thread_mode,
    );

    ctx.get_mut_symbols().new_static(
        name,
        (static_type.clone(), metadata, attributes.clone()),
        span,
    )?;

    Ok(Ast::Static {
        name,
        ascii_name,
        kind: static_type,
        value: Some(value.into()),
        attributes,
        metadata,
        span,
    })
}
