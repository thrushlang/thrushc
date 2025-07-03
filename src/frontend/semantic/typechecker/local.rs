use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds, position::TypeCheckerPosition},
        types::{
            ast::{Ast, metadata::local::LocalMetadata},
            lexer::Type,
        },
    },
};

pub fn validate_local<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Local {
            name,
            kind: local_type,
            value: local_value,
            span,
            metadata,
            ..
        } => {
            typechecker.symbols.new_local(name, local_type);

            let metadata: &LocalMetadata = metadata;

            if local_type.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            if !metadata.is_undefined() {
                let local_value_type: &Type = local_value.get_value_type()?;

                if let Err(error) = bounds::checking::check(
                    local_type,
                    local_value_type,
                    Some(local_value),
                    None,
                    Some(TypeCheckerPosition::Local),
                    span,
                ) {
                    typechecker.add_error(error);
                }

                if let Err(type_error) = typechecker.analyze_ast(local_value) {
                    typechecker.add_error(type_error);
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
