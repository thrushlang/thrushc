use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstGetType, AstStandardExtensions};
use crate::front_end::typesystem::{traits::TypePointerExtensions, types::Type};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::LLI {
            name,
            kind: lli_type,
            expr,
            span,
            ..
        } => {
            typechecker.symbols.new_lli(name, (lli_type, *span));

            let expr_span: Span = expr.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(expr.is_literal_value(), expr_span);

            let value_type: &Type = expr.get_value_type()?;

            if lli_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            checks::check_types(lli_type, value_type, Some(expr), None, metadata)?;

            typechecker.analyze_expr(expr)?;

            Ok(())
        }

        Ast::Load { source, .. } => {
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if !source_type.is_ptr_type() && !source_type.is_address_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!(
                        "Expected raw typed pointer 'ptr[T]', pointer 'ptr' or memory address 'addr' type, got '{}'.",
                        source_type
                    ),
                    None,
                    span,
                ));
            }

            typechecker.analyze_expr(source)?;

            Ok(())
        }

        Ast::Address {
            source, indexes, ..
        } => {
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if !source_type.is_ptr_type() && !source_type.is_address_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!(
                        "Expected raw typed pointer 'ptr[T]', pointer 'ptr' or memory address 'addr' type, got '{}'.",
                        source_type
                    ),
                    None,
                    span,
                ));
            }

            if source_type.is_ptr_type() && !source_type.is_typed_ptr_type() && indexes.len() > 1 {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!(
                        "Expected raw typed pointer ptr[T] instead, got '{}'.",
                        source_type
                    ),
                    None,
                    span,
                ));
            } else if source_type.is_ptr_type()
                && source_type.is_typed_ptr_type()
                && !source_type.is_ptr_indexable_like_type()
            {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!("Expected raw typed pointer type with indexable type 'struct T', or 'array[T; N]', got '{}'.", source_type),
                    None,
                    span,
                ));
            }

            indexes.iter().try_for_each(|indexe| {
                let indexe_type: &Type = indexe.get_value_type()?;
                let span: Span = indexe.get_span();

                if !indexe_type.is_unsigned_integer_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        "Type error".into(),
                        format!("Expected unsigned integer type, got '{}'.", indexe_type),
                        None,
                        span,
                    ));
                }

                typechecker.analyze_expr(indexe)?;

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
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                     format!(
                        "Expected raw typed pointer 'ptr[T]', pointer 'ptr' or memory address 'addr' type, got '{}'.",
                        source_type
                    ),
                    None,
                    span,
                ));
            }

            typechecker.analyze_expr(source)?;

            let value_type: &Type = write_value.get_value_type()?;
            let span: Span = write_value.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(write_value.is_literal_value(), span);

            checks::check_types(write_type, value_type, Some(write_value), None, metadata)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(CompilationIssue::FrontEndBug(
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
