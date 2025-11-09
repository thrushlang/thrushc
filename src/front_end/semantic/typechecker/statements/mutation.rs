use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

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
                TypeCheckerExprMetadata::new(value.is_literal(), *span);

            let value_type: &Type = value.get_value_type()?;
            let source_type: &Type = source.get_value_type()?;

            if !source.is_allocated() && !source_type.is_ptr_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer 'ptr[T]', raw pointer 'ptr' type or allocated reference.".into(),
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

            if !source_type.is_ptr_type() {
                let lhs_type: &Type = source_type;
                let rhs_type: &Type = value_type;

                checks::check_types(lhs_type, rhs_type, Some(value), None, metadata)?;
            }

            typechecker.analyze_expr(value)?;

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
