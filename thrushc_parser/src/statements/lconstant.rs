use thrushc_ast::{Ast, metadata::ConstantMetadata};
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_mir::atomicord::ThrushAtomicOrdering;
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, builder, expressions, typegen};

pub fn build_const<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Const,
        CompilationIssueCode::E0001,
        "Expected 'const' keyword.".into(),
    )?;

    let is_lazy: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<ThrushAtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let const_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected name.".into(),
    )?;

    let name: &str = const_tk.get_lexeme();
    let ascii_name: &str = const_tk.get_ascii_lexeme();

    let span: Span = const_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        "Expected ':'.".into(),
    )?;

    let const_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        "Expected '='.".into(),
    )?;

    let value: Ast = expressions::build_expression(ctx)?;

    let metadata: ConstantMetadata = ConstantMetadata::new(false, is_lazy, is_volatile, atom_ord);

    ctx.get_mut_symbols()
        .new_constant(name, (const_type.clone(), attributes.clone()), span)?;

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes,
        metadata,
        span,
    })
}
