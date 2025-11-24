use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::EntryPoint {
            body, kind, span, ..
        } => {
            if !kind.is_signed_integer_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    format!("Expected signed integer, got '{}'.", kind),
                    None,
                    *span,
                ));
            }

            typechecker.analyze_stmt(body)?;

            if !body.has_terminator() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    format!("Expected return with type '{}'.", kind),
                    None,
                    *span,
                ));
            }

            Ok(())
        }

        Ast::AssemblerFunction { parameters, .. } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_value_type()?.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            Ok(())
        }

        Ast::Intrinsic { parameters, .. } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_value_type()?.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            Ok(())
        }

        Ast::Function {
            parameters,
            body,
            return_type,
            span,
            ..
        } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_any_type()?.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            if let Some(body) = body {
                typechecker.analyze_stmt(body)?;

                if !body.has_terminator() && !return_type.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        format!("Expected return with type '{}'.", return_type),
                        None,
                        *span,
                    ));
                }
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
