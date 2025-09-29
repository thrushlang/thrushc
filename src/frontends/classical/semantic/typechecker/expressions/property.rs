use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::ast::Ast,
        typesystem::{
            traits::{TypeMutableExtensions, TypePointerExtensions},
            types::Type,
        },
    },
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Property { source, .. } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;

                let reference_type: &Type = reference.get_value_type()?;
                let reference_span: Span = reference.get_span();

                if !reference_type.is_struct_type()
                    && !reference_type.is_mut_struct_type()
                    && !reference_type.is_ptr_struct_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "A structure type was expected within the high-level pointer 'mut T', the raw typed pointer 'mut T', or a structure 'struct T'.".into(),
                        None,
                        reference_span,
                    ));
                }

                typechecker.analyze_stmt(reference)?;

                return Ok(());
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_value_type()?;
                let expr_span: Span = expr.get_span();

                if !expr_type.is_struct_type()
                    && !expr_type.is_mut_struct_type()
                    && !expr_type.is_ptr_struct_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "A structure type was expected within the high-level pointer 'mut T', the raw typed pointer 'mut T', or a structure 'struct T'.".into(),
                        None,
                        expr_span,
                    ));
                }

                typechecker.analyze_stmt(expr)?;

                return Ok(());
            }

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
