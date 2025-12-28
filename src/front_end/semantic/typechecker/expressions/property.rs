use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssueCode;
use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstGetType};
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::{traits::TypePointerExtensions, types::Type};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Property { source, .. } => {
            let source_type: &Type = source.get_value_type()?;
            let source_span: Span = source.get_span();

            if !source_type.is_struct_type() && !source_type.is_ptr_struct_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    format!("A structure type was expected within a structure 'struct T' type, or raw typed pointer structure pointer 'ptr[struct T]', got '{}'.", source_type),
                    None,
                    source_span,
                ));
            }

            typechecker.analyze_expr(source)?;

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
