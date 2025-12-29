use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::checks;
use crate::front_end::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
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
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "Void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            if let Some(local_value) = value {
                let metadata: &LocalMetadata = metadata;

                let type_metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(local_value.is_literal_value());

                if local_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "Void type isn't a value.".into(),
                        None,
                        *span,
                    ));
                }

                if !metadata.is_undefined() {
                    let local_value_type: &Type = local_value.get_value_type()?;

                    checks::check_types(
                        local_type,
                        local_value_type,
                        Some(local_value),
                        None,
                        type_metadata,
                        node.get_span(),
                    )?;

                    typechecker.analyze_expr(local_value)?;
                }
            }

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
