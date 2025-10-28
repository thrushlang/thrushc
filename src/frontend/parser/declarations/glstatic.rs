use inkwell::{AtomicOrdering, ThreadLocalMode};

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::attributes;
use crate::frontend::parser::builder;
use crate::frontend::parser::expr;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::types::Type;

pub fn build_global_static<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::Static,
        "Syntax error".into(),
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;
    let thread_local: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atomic_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;
    let thread_mode: Option<ThreadLocalMode> = builder::build_thread_local_mode(ctx)?;

    let static_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = static_tk.get_lexeme();
    let ascii_name: &str = static_tk.get_ascii_lexeme();

    let span: Span = static_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let static_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: StaticMetadata = StaticMetadata::new(
            true,
            is_mutable,
            true,
            thread_local,
            is_volatile,
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

    ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(ctx)?;

    let metadata: StaticMetadata = StaticMetadata::new(
        true,
        is_mutable,
        false,
        thread_local,
        is_volatile,
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
