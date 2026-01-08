use thrushc_ast::{Ast, metadata::ConstantMetadata};
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_mir::atomicord::ThrushAtomicOrdering;
use thrushc_modificators::{Modificators, traits::ModificatorsExtensions};
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

    let modificators: Modificators =
        builder::build_stmt_modificator(ctx, &[TokenType::Identifier])?;

    let thread_local: bool = modificators.has_lazythread();
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrushAtomicOrdering> = modificators.get_atomic_ordering();

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

    let metadata: ConstantMetadata =
        ConstantMetadata::new(false, thread_local, is_volatile, atomic_ord);

    ctx.get_mut_symbols()
        .new_constant(name, (const_type.clone(), attributes.clone()), span)?;

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes,
        modificators,
        metadata,
        span,
    })
}
