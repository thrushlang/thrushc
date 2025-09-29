use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
        types::ast::Ast,
        typesystem::{traits::TypePointerExtensions, types::Type},
    },
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::LLI {
            name,
            kind: lli_type,
            value,
            span,
            ..
        } => {
            typechecker.symbols.new_lli(name, (lli_type, *span));

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(value.is_literal(), None, *span);

            let value_type: &Type = value.get_value_type()?;

            if lli_type.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            if let Err(error) =
                checks::type_check(lli_type, value_type, Some(value), None, metadata)
            {
                typechecker.add_error(error);
            }

            typechecker.analyze_stmt(value)?;

            Ok(())
        }

        Ast::Load { source, .. } => {
            if let Some(left) = &source.0 {
                let reference: &Ast = &left.1;

                let reference_type: &Type = reference.get_value_type()?;
                let span: Span = reference.get_span();

                if !reference_type.is_ptr_type() && !reference_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_stmt(reference)?;
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_value_type()?;
                let span: Span = expr.get_span();

                if !expr_type.is_ptr_type() && !expr_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr' or 'addr' type.".into(),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_stmt(expr)?;
            }

            Ok(())
        }

        Ast::Address {
            source,
            indexes,
            span,
            ..
        } => {
            if let Some(reference_any) = &source.0 {
                let reference: &Ast = &reference_any.1;

                let reference_type: &Type = reference.get_value_type()?;
                let span: Span = reference.get_span();

                if !reference_type.is_ptr_type() && !reference_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        span,
                    ));
                }

                if reference_type.is_ptr_type() && !reference_type.is_typed_ptr_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        span,
                    ));
                } else if reference_type.is_ptr_type()
                    && reference_type.is_typed_ptr_type()
                    && !reference_type.is_ptr_struct_type()
                    && !reference_type.is_ptr_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type 'struct T', or 'array[T; N]'."
                            .into(),
                        None,
                        span,
                    ));
                }
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_value_type()?;
                let span: Span = expr.get_span();

                if !expr_type.is_ptr_type() && !expr_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        span,
                    ));
                }

                if expr_type.is_ptr_type() && !expr_type.is_typed_ptr_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        span,
                    ));
                } else if expr_type.is_ptr_type()
                    && expr_type.is_typed_ptr_type()
                    && !expr_type.is_ptr_struct_type()
                    && !expr_type.is_ptr_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type 'struct T', or 'array[T; N]'.".into(),
                        None,
                        span,
                    ));
                }
            }

            indexes.iter().try_for_each(|indexe| {
                if !indexe.is_unsigned_integer()? {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected any unsigned integer value.".into(),
                        None,
                        *span,
                    ));
                }

                typechecker.analyze_stmt(indexe)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Write {
            source,
            write_value,
            write_type,
            ..
        } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                let reference_type: &Type = reference.get_value_type()?;
                let span: Span = reference.get_span();

                if !reference_type.is_ptr_type()
                    && !reference_type.is_address_type()
                    && !reference_type.is_mut_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', 'addr', or 'mut T' type.".into(),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_stmt(reference)?;
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_value_type()?;
                let span: Span = expr.get_span();

                if !expr_type.is_ptr_type()
                    && !expr_type.is_address_type()
                    && !expr_type.is_mut_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', 'addr', or 'mut T' type.".into(),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_stmt(expr)?;
            }

            let value_type: &Type = write_value.get_value_type()?;
            let span: Span = write_value.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(write_value.is_literal(), None, span);

            if let Err(error) =
                checks::type_check(write_type, value_type, Some(write_value), None, metadata)
            {
                typechecker.add_error(error);
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
