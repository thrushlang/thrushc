use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::builder;
use crate::front_end::parser::expressions;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::staticvar::StaticMetadata;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use inkwell::{AtomicOrdering, ThreadLocalMode};

pub fn build_global_static<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Static,
        CompilationIssueCode::E0001,
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;
    let thread_local: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atomic_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;
    let thread_mode: Option<ThreadLocalMode> = builder::build_thread_local_mode(ctx)?;

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

        if declare_forward {
            ctx.get_mut_symbols().new_global_static(
                name,
                (static_type.clone(), metadata, attributes.clone()),
                span,
            )?;
        }

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

    if declare_forward {
        ctx.get_mut_symbols().new_global_static(
            name,
            (static_type.clone(), metadata, attributes.clone()),
            span,
        )?;
    }

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
