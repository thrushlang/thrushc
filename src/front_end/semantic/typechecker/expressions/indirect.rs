use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::expressions;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Indirect {
            function,
            function_type,
            args,
            span,
            ..
        } => {
            let function_ref: &Type = function.get_value_type()?;

            if !function_ref.is_fnref_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!(
                        "Expected function reference 'Fn[..] -> T' type, got '{}'.",
                        function_ref
                    ),
                    None,
                    *span,
                ));
            }

            if let Type::Fn(parameter_types, _, modificator) = function_type {
                expressions::call::validate(
                    typechecker,
                    (parameter_types, modificator.llvm().has_ignore()),
                    args,
                    span,
                )?;
            } else {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    format!(
                        "Expected function reference 'Fn[..] -> T' type, got '{}'.",
                        function_type
                    ),
                    None,
                    *span,
                ));
            }

            args.iter()
                .try_for_each(|arg| typechecker.analyze_expr(arg))?;

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
