use {
    crate::{
        core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
        frontends::classical::{
            lexer::span::Span,
            semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
            types::ast::Ast,
            typesystem::types::Type,
        },
    },
    std::{fmt::Write, path::PathBuf},
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Indirect {
            pointer,
            function_type,
            args,
            span,
            ..
        } => {
            let function_pointer_type: &Type = pointer.get_value_type()?;

            if !function_pointer_type.is_fnref_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected function reference 'Fn[..] -> T' type.".into(),
                    None,
                    *span,
                ));
            }

            if let Type::Fn(parameter_types, ..) = function_type {
                let required_size: usize = parameter_types.len();
                let provided_size: usize = args.len();

                let mut types_display: String = String::with_capacity(100);

                parameter_types.iter().for_each(|parameter_type| {
                    let _ = write!(types_display, "{}", parameter_type);
                });

                if required_size != provided_size {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        format!(
                            "Expected arguments total '{}', not '{}'.",
                            required_size, provided_size
                        ),
                        None,
                        *span,
                    ));

                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        format!("Arguments were expected in the order: '{}'.", types_display),
                        None,
                        *span,
                    ));

                    return Ok(());
                }

                parameter_types
                    .iter()
                    .zip(args.iter())
                    .try_for_each(|(target_type, expr)| {
                        let from_type: &Type = expr.get_value_type()?;
                        let span: Span = expr.get_span();

                        let metadata: TypeCheckerExprMetadata =
                            TypeCheckerExprMetadata::new(expr.is_literal(), None, span);

                        if let Err(error) =
                            checks::type_check(target_type, from_type, Some(expr), None, metadata)
                        {
                            typechecker.add_error(error);
                        }

                        Ok(())
                    })?;
            }

            args.iter()
                .try_for_each(|arg| typechecker.analyze_stmt(arg))?;

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
