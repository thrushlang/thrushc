use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::{
            TypeChecker, checks, metadata::TypeCheckerExprMetadata, position::TypeCheckerPosition,
        },
        types::ast::{Ast, metadata::local::LocalMetadata},
        typesystem::types::Type,
    },
};

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

                let type_metadata: TypeCheckerExprMetadata = TypeCheckerExprMetadata::new(
                    local_value.is_literal(),
                    Some(TypeCheckerPosition::Local),
                    *span,
                );

                if local_type.is_void_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The void type isn't a value.".into(),
                        None,
                        *span,
                    ));
                }

                if !metadata.is_undefined() {
                    let mut local_value_type: Type = local_value.get_value_type()?.clone();

                    if local_value_type.is_ptr_type() {
                        local_value_type = Type::Ptr(Some(local_value_type.into()));
                    }

                    checks::check_types(
                        local_type,
                        &local_value_type,
                        Some(local_value),
                        None,
                        type_metadata,
                    )?;

                    if let Err(type_error) = typechecker.analyze_stmt(local_value) {
                        typechecker.add_error(type_error);
                    }
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
