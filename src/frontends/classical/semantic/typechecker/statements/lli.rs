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
            expr,
            span,
            ..
        } => {
            typechecker.symbols.new_lli(name, (lli_type, *span));

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(expr.is_literal(), *span);

            let value_type: &Type = expr.get_value_type()?;

            if lli_type.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            checks::check_types(lli_type, value_type, Some(expr), None, metadata)?;

            typechecker.analyze_stmt(expr)?;

            Ok(())
        }

        Ast::Load { source, .. } => {
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if !source_type.is_ptr_type() && !source_type.is_address_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected 'ptr<T>', 'ptr' or 'addr' type.".into(),
                    None,
                    span,
                ));
            }

            typechecker.analyze_stmt(source)?;

            Ok(())
        }

        Ast::Address {
            source, indexes, ..
        } => {
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if !source_type.is_ptr_type() && !source_type.is_address_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                    None,
                    span,
                ));
            }

            if source_type.is_ptr_type() && !source_type.is_typed_ptr_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer ptr<T>.".into(),
                    None,
                    span,
                ));
            } else if source_type.is_ptr_type()
                && source_type.is_typed_ptr_type()
                && !source_type.is_ptr_struct_type()
                && !source_type.is_ptr_fixed_array_type()
            {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer type with deep type 'struct T', or 'array[T; N]'."
                        .into(),
                    None,
                    span,
                ));
            }

            indexes.iter().try_for_each(|indexe| {
                let span: Span = indexe.get_span();

                if !indexe.is_unsigned_integer_for_index()? {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected any unsigned integer value.".into(),
                        None,
                        span,
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
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if !source_type.is_ptr_type() && !source_type.is_address_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected 'ptr<T>', 'ptr', 'addr' type.".into(),
                    None,
                    span,
                ));
            }

            typechecker.analyze_stmt(source)?;

            let value_type: &Type = write_value.get_value_type()?;
            let span: Span = write_value.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(write_value.is_literal(), span);

            checks::check_types(write_type, value_type, Some(write_value), None, metadata)?;

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
