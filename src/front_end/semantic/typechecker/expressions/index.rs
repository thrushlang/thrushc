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
            index,
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
                    format!("Expected raw typed pointer ptr[T], raw pointer ptr, array[T], or fixed array[T; N], got '{}'.", source_type),
                    None,
                    *span,
                ));
            }

            if source.is_reference() && !source.is_allocated() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    format!("Expected raw typed pointer ptr[T], raw pointer ptr, array[T], or fixed array[T; N], got '{}'.", source_type),
                    None,
                    *span,
                ));
            }

            let index_type: &Type = index.get_value_type()?;
            let span: Span = index.get_span();

            if !index_type.is_integer_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    format!("Expected integer value, got '{}'.", index_type),
                    None,
                    span,
                ));
            }

            typechecker.analyze_expr(index)?;
            typechecker.analyze_expr(source)?;

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
