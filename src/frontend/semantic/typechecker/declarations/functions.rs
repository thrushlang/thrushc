use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::typechecker::TypeChecker, types::ast::Ast},
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::EntryPoint { body, span, .. } => {
            if let Err(type_error) = typechecker.analyze_stmt(body) {
                typechecker.add_error(type_error);
            }

            if !body.has_return_for_function() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected return with type 'u32'.".into(),
                    None,
                    *span,
                ));
            }

            Ok(())
        }

        Ast::AssemblerFunction {
            parameters, span, ..
        } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_value_type()?.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        *span,
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
                        *span,
                    ));
                }

                Ok(())
            })?;

            if body.is_block() {
                if let Err(type_error) = typechecker.analyze_stmt(body) {
                    typechecker.add_error(type_error);
                }

                if !body.has_return_for_function() && !return_type.is_void_type() {
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
                line!(),
            ));

            Ok(())
        }
    }
}
