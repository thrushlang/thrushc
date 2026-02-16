use thrustc_ast::{Ast, metadata::ConstantMetadata};
use thrustc_attributes::ThrustAttributes;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_mir::atomicord::ThrustAtomicOrdering;
use thrustc_modificators::{Modificators, traits::ModificatorsExtensions};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{ParserContext, attributes, expressions, modificators, typegen};

pub fn build_global_const<'parser>(
    ctx: &mut ParserContext<'parser>,
    parse_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Const,
        CompilationIssueCode::E0001,
        "Expected 'const' keyword.".into(),
    )?;

    let modificators: Modificators =
        modificators::build_stmt_modificator(ctx, &[TokenType::Identifier])?;

    let thread_local: bool = modificators.has_lazythread();
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrustAtomicOrdering> = modificators.get_atomic_ordering();

    let const_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
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

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        "Expected '='.".into(),
    )?;

    let value: Ast = expressions::build_expression(ctx)?;

    let metadata: ConstantMetadata =
        ConstantMetadata::new(true, thread_local, is_volatile, atomic_ord);

    if parse_forward {
        ctx.get_mut_symbols()
            .new_global_constant(name, (const_type, attributes), span)?;

        Ok(Ast::new_nullptr(span))
    } else {
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
}
