use thrushc_ast::{
    Ast,
    traits::{AstCodeBlockEntensions, AstCodeLocation, AstGetType},
};
use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};
use thrushc_span::Span;
use thrushc_typesystem::traits::TypeIsExtensions;

use crate::TypeChecker;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::AssemblerFunction { parameters, .. } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_value_type()?.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "Void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            Ok(())
        }

        Ast::Intrinsic { parameters, .. } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_value_type()?.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "Void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            Ok(())
        }

        Ast::Function {
            parameters,
            body,
            return_type,
            span,
            ..
        } => {
            parameters.iter().try_for_each(|parameter| {
                if parameter.get_any_type()?.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "Void type isn't a value.".into(),
                        None,
                        parameter.get_span(),
                    ));
                }

                Ok(())
            })?;

            if let Some(body) = body {
                typechecker.analyze_stmt(body)?;

                if !body.has_terminator() && !return_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        format!("Expected return with type '{}'.", return_type),
                        None,
                        *span,
                    ));
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
                std::path::PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
