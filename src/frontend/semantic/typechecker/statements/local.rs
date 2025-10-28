use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::frontend::types::ast::{Ast, metadata::local::LocalMetadata};
use crate::frontend::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Local {
            name,
            kind: local_type,
            value,
            span,
            metadata,
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

            if let Some(local_value) = value {
                let metadata: &LocalMetadata = metadata;

                let type_metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(local_value.is_literal(), *span);

                if local_type.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        *span,
                    ));
                }

                if !metadata.is_undefined() {
                    let local_value_type: Type = local_value.get_value_type()?.clone();

                    checks::check_types(
                        local_type,
                        &local_value_type,
                        Some(local_value),
                        None,
                        type_metadata,
                    )?;

                    typechecker.analyze_stmt(local_value)?;
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
