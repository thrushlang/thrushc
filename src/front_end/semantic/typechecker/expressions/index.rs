use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Index {
            source,
            indexes,
            span,
            ..
        } => {
            let source_type: &Type = source.get_any_type()?;

            if !source_type.is_ptr_type()
                && !source_type.is_array_type()
                && !source_type.is_fixed_array_type()
            {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer ptr[T], raw pointer ptr, array[T], or fixed array[T; N].".into(),
                    None,
                    *span,
                ));
            }

            indexes.iter().try_for_each(|indexe| {
                let indexe_type: &Type = indexe.get_value_type()?;
                let span: Span = indexe.get_span();

                if !indexe_type.is_integer_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected integer value.".into(),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_expr(indexe)
            })?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(ThrushCompilerIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
