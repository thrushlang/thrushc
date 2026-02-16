use thrustc_ast::{Ast, metadata::StaticMetadata};
use thrustc_attributes::{ThrustAttributes, traits::ThrustAttributesExtensions};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_mir::{atomicord::ThrustAtomicOrdering, threadmode::ThrustThreadMode};
use thrustc_modificators::{Modificators, traits::ModificatorsExtensions};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{ParserContext, attributes, expressions, modificators, typegen};

pub fn build_global_static<'parser>(
    ctx: &mut ParserContext<'parser>,
    parse_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Static,
        CompilationIssueCode::E0001,
        "Expected 'static' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

    let modificators: Modificators =
        modificators::build_stmt_modificator(ctx, &[TokenType::Identifier])?;

    let thread_local: bool = modificators.has_lazythread();
    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrustAtomicOrdering> = modificators.get_atomic_ordering();
    let thread_mode: Option<ThrustThreadMode> = modificators.get_thread_mode();

    let static_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
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

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::Eq])?;
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

        if parse_forward {
            ctx.get_mut_symbols().new_global_static(
                name,
                (static_type, metadata, attributes),
                span,
            )?;

            Ok(Ast::new_nullptr(span))
        } else {
            let static_: Ast = Ast::Static {
                name,
                ascii_name,
                kind: static_type,
                value: None,
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(static_)
        }
    } else {
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

        if parse_forward {
            ctx.get_mut_symbols().new_global_static(
                name,
                (static_type, metadata, attributes),
                span,
            )?;

            Ok(Ast::new_nullptr(span))
        } else {
            let static_: Ast = Ast::Static {
                name,
                ascii_name,
                kind: static_type,
                value: Some(value.into()),
                attributes,
                modificators,
                metadata,
                span,
            };

            Ok(static_)
        }
    }
}
