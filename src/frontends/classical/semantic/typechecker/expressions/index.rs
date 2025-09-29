use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::ast::Ast,
        typesystem::{traits::TypeMutableExtensions, types::Type},
    },
};

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
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;

                if !reference.is_allocated() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected memory reference, such as ptr[T], ptr, addr, high-level pointer mut T or direct reference to variable.".into(),
                        None,
                        *span,
                    ));
                }

                let reference_type: &Type = reference.get_value_type()?;

                if !reference_type.is_ptr_type()
                    && !reference_type.is_mut_array_type()
                    && !reference_type.is_mut_fixed_array_type()
                    && !reference_type.is_array_type()
                    && !reference_type.is_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr[T], raw pointer ptr, array[T], or fixed array[T; N].".into(),
                        None,
                        *span,
                    ));
                }
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_any_type()?;

                if !expr_type.is_ptr_type()
                    && !expr_type.is_mut_array_type()
                    && !expr_type.is_mut_fixed_array_type()
                    && !expr_type.is_array_type()
                    && !expr_type.is_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr[T], raw pointer ptr, array[T], or fixed array[T; N].".into(),
                        None,
                        *span,
                    ));
                }
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

                typechecker.analyze_stmt(indexe)
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
