use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds},
        types::ast::{Ast, traits::LLVMAstExtensions},
        typesystem::types::Type,
    },
};

pub fn validate_static<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Static {
            kind: target_type,
            value,
            span,
            ..
        } => {
            let from_type: &Type = value.get_value_type()?;
            let expression_span: Span = value.get_span();

            if !value.is_llvm_constant_value() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected integer, floating-point, boolean, string, fixed array, or char constant types.".into(),
                    None,
                    expression_span,
                ));
            }

            if let Err(error) =
                bounds::checking::check(target_type, from_type, Some(value), None, None, span)
            {
                typechecker.add_error(error);
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
