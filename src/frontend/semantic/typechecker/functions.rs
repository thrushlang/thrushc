use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_function<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::EntryPoint { body, .. } => {
            if let Err(type_error) = typechecker.analyze_stmt(body) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        ThrushStatement::AssemblerFunction {
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

        ThrushStatement::Function {
            parameters,
            body,
            return_type,
            span,
            ..
        } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_stmt_type()?.is_void_type() {
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

                if !body.has_return() {
                    if let Err(mismatch_type_error) = typechecker.validate_types(
                        return_type,
                        &ThrushType::Void,
                        None,
                        None,
                        None,
                        span,
                    ) {
                        typechecker.add_error(mismatch_type_error);
                    }
                }
            }

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(ThrushCompilerIssue::Bug(
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
