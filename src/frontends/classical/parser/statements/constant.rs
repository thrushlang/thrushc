use inkwell::AtomicOrdering;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, builder, checks, expr, typegen},
        types::{
            ast::{Ast, metadata::constant::ConstantMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_const<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.consume(
        TokenType::Const,
        "Syntax error".into(),
        "Expected 'const' keyword.".into(),
    )?;

    let is_lazy: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let const_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = const_tk.get_lexeme();
    let ascii_name: &str = const_tk.get_ascii_lexeme();

    let span: Span = const_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let const_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(ctx)?;

    if let Err(error) =
        ctx.get_mut_symbols()
            .new_constant(name, (const_type.clone(), attributes.clone()), span)
    {
        ctx.add_silent_error(error);
    }

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes,
        metadata: ConstantMetadata::new(false, is_lazy, is_volatile, atom_ord),
        span,
    })
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)?;
    checks::check_inside_function_state(ctx)
}
