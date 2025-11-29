use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::checks;
use crate::front_end::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstStandardExtensions;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Return {
            expression, kind, ..
        } => {
            if let Some(expr) = expression {
                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(expr.is_literal_value(), expr.get_span());

                checks::check_types(kind, expr.get_value_type()?, Some(expr), None, metadata)?;

                typechecker.analyze_expr(expr)?;
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
