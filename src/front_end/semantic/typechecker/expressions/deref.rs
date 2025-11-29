use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Deref { value, .. } => {
            let value_type: &Type = value.get_value_type()?;

            if !value_type.is_ptr_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!("Expected raw typed pointer 'ptr[T]' type, raw pointer 'ptr' type for defererence, got '{}'.", value_type),
                    None,
                    value.get_span(),
                ));
            }

            typechecker.analyze_expr(value)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(CompilationIssue::FrontEndBug(
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
