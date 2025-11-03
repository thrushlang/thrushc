use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::typechecker::TypeChecker;
use crate::frontend::semantic::typechecker::checks;
use crate::frontend::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Enum { fields, .. } => {
            fields.iter().try_for_each(|field| {
                let target_type: Type = field.1.clone();
                let value: &Ast = &field.2;
                let span: Span = value.get_span();

                let from_type: &Type = value.get_value_type()?;

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(value.is_literal(), span);

                checks::check_types(&target_type, from_type, Some(value), None, metadata)?;

                typechecker.analyze_expr(value)?;

                Ok(())
            })?;

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
