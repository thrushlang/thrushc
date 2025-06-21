use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, position::TypeCheckerPosition},
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_local<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::Local {
            name,
            kind: local_type,
            value: local_value,
            span,
            undefined,
            ..
        } => {
            typechecker.symbols.new_local(name, local_type);

            if local_type.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            if local_type.is_ptr_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Raw pointer type 'ptr<T>', or 'ptr' can only be used in Low Level Instructions (LLI), use them instead.".into(),
                    None,
                    *span,
                ));
            }

            if !*undefined {
                let local_value_type: &ThrushType = local_value.get_value_type()?;

                if let Err(mismatch_type_error) = typechecker.validate_types(
                    local_type,
                    local_value_type,
                    Some(local_value),
                    None,
                    Some(TypeCheckerPosition::Local),
                    span,
                ) {
                    typechecker.add_error(mismatch_type_error);
                }

                if let Err(type_error) = typechecker.analyze_stmt(local_value) {
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
