use thrustc_ast::{Ast, metadata::DereferenceMetadata, traits::AstGetType};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_mir::atomicord::ThrustAtomicOrdering;
use thrustc_modificators::{Modificators, traits::ModificatorsExtensions};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::{Type, traits::DereferenceExtensions};

use crate::{ParserContext, expressions, modificators};

pub fn build_dereference<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let initial_deref_tk: &Token = ctx.advance()?;
    let span: Span = initial_deref_tk.get_span();

    let modificators: Modificators =
        modificators::build_stmt_modificator(ctx, &[TokenType::Identifier])?;

    let is_volatile: bool = modificators.has_volatile();
    let atomic_ord: Option<ThrustAtomicOrdering> = modificators.get_atomic_ordering();

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
            modificators: modificators.clone(),
            metadata: DereferenceMetadata::new(is_volatile, atomic_ord),
            span,
        };

        current_type = current_type.dereference();
    });

    Ok(current_expr)
}
