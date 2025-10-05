use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span, semantic::typechecker::TypeChecker, types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Defer { value, .. } => {
            let value_type: &Type = value.get_value_type()?;
            let value_span: Span = value.get_span();

            if !value_type.is_ptr_type()
                && !value_type.is_array_type()
                && !value_type.is_const_type()
            {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected raw typed pointer 'ptr[T]' type, raw pointer 'ptr' type, array type 'array[T]', or constant 'const T' type for defererence."
                        .into(),
                    None,
                    value_span,
                ));
            }

            typechecker.analyze_stmt(value)?;

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
