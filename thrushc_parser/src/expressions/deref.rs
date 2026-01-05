use thrushc_ast::{Ast, metadata::DereferenceMetadata, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_mir::atomicord::ThrushAtomicOrdering;
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType};
use thrushc_typesystem::{Type, traits::DereferenceExtensions};

use crate::{ParserContext, builder, expressions};

pub fn build_dereference<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let initial_deref_tk: &Token = ctx.advance()?;
    let span: Span = initial_deref_tk.span;

    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;
    let atomic_ord: Option<ThrushAtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let mut deref_count: u64 = 1;

    let mut current_expr: Ast = {
        while ctx.check(TokenType::Deref) {
            ctx.consume(
                TokenType::Deref,
                CompilationIssueCode::E0001,
                "Expected 'deref' keyword.".into(),
            )?;

            deref_count += 1;
        }

        let expr: Ast = expressions::build_expr(ctx)?;

        expr
    };

    let mut current_type: Type = current_expr.get_value_type()?.clone();

    (0..deref_count).for_each(|_| {
        current_expr = Ast::Deref {
            value: current_expr.clone().into(),
            kind: current_type.dereference(),
            metadata: DereferenceMetadata::new(is_volatile, atomic_ord),
            span,
        };

        current_type = current_type.dereference();
    });

    Ok(current_expr)
}
