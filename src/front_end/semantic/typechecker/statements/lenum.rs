use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::checks;
use crate::front_end::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Enum { fields, .. } => {
            fields.iter().try_for_each(|field| {
                let target_type: Type = field.1.clone();
                let from_type: &Type = field.2.get_value_type()?;

                let value: &Ast = &field.2;

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(value.is_literal_value(), value.get_span());

                checks::check_types(&target_type, from_type, Some(value), None, metadata)?;

                typechecker.analyze_expr(value)?;

                Ok(())
            })?;

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
