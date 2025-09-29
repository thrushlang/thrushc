use inkwell::{AtomicOrdering, ThreadLocalMode};

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, builder, expr, typegen},
        types::{
            ast::{Ast, metadata::staticvar::StaticMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

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
    let is_lazy: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;
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

    ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(ctx)?;

    let metadata: StaticMetadata = StaticMetadata::new(
        true,
        is_mutable,
        is_lazy,
        is_volatile,
        atom_ord,
        thread_mode,
    );

    if declare_forward {
        if let Err(error) = ctx.get_mut_symbols().new_global_static(
            name,
            (static_type.clone(), metadata, attributes.clone()),
            span,
        ) {
            ctx.add_silent_error(error);
        }
    }

    Ok(Ast::Static {
        name,
        ascii_name,
        kind: static_type,
        value: value.into(),
        attributes,
        metadata,
        span,
    })
}
