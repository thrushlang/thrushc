use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
        types::ast::Ast,
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Mut {
            source,
            value,
            span,
            ..
        } => {
            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(value.is_literal(), None, *span);

            let value_type: &Type = value.get_value_type()?;
            let source_type: &Type = source.get_value_type()?;

            if !source.is_allocated() && !source_type.is_ptr_type() && !source_type.is_mut_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', or high-level pointer 'mut T' type."
                        .into(),
                    None,
                    *span,
                ));
            }

            if !source.is_mutable() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The reference must be marked as mutable.".into(),
                    None,
                    source.get_span(),
                ));
            }

            if source_type.is_mut_type() {
                let lhs_type: &Type = source_type.get_type_with_depth(1);
                let rhs_type: &Type = value_type;

                if let Err(error) = checks::type_check(lhs_type, rhs_type, None, None, metadata) {
                    typechecker.add_error(error);
                }
            }

            if source_type.is_ptr_type() {
                let lhs_type: &Type = source_type.get_type_with_depth(1);
                let rhs_type: &Type = value_type;

                if let Err(error) = checks::type_check(lhs_type, rhs_type, None, None, metadata) {
                    typechecker.add_error(error);
                }
            }

            typechecker.analyze_stmt(value)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(ThrushCompilerIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                line!(),
            ));

            Ok(())
        }
    }
}
