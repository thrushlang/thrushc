use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::{traits::TypePointerExtensions, types::Type};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Property { source, .. } => {
            let source_type: &Type = source.get_value_type()?;
            let source_span: Span = source.get_span();

            if !source_type.is_struct_type() && !source_type.is_ptr_struct_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "A structure type was expected within the raw typed pointer 'ptr[T]', or a structure 'struct T'.".into(),
                    None,
                    source_span,
                ));
            }

            typechecker.analyze_expr(source)?;

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
