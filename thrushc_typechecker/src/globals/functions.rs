use thrushc_ast::{
    Ast,
    traits::{AstCodeBlockEntensions, AstCodeLocation, AstGetType},
};

use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};
use thrushc_span::Span;
use thrushc_typesystem::{Type, traits::TypeIsExtensions};

use crate::TypeChecker;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::AssemblerFunction {
            name,
            parameters,
            parameters_types,
            return_type,
            attributes,
            ..
        } => {
            if !typechecker.get_table().constains_asm_function(name) {
                typechecker
                    .get_mut_table()
                    .new_asm_function(name, (return_type, parameters_types, attributes));
            }

            {
                for node in parameters.iter() {
                    let type_: &Type = node.get_any_type()?;
                    let span: Span = node.get_span();

                    if type_.is_void_type() {
                        typechecker.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0019,
                            "Void type isn't a value.".into(),
                            None,
                            span,
                        ));
                    }
                }
            }

            Ok(())
        }

        Ast::Intrinsic {
            name,
            parameters,
            parameters_types,
            return_type,
            attributes,
            ..
        } => {
            if !typechecker.get_table().constains_intrinsic(name) {
                typechecker
                    .get_mut_table()
                    .new_intrinsic(name, (return_type, parameters_types, attributes));
            }

            {
                for node in parameters.iter() {
                    let type_: &Type = node.get_any_type()?;
                    let span: Span = node.get_span();

                    if type_.is_void_type() {
                        typechecker.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0019,
                            "Void type isn't a value.".into(),
                            None,
                            span,
                        ));
                    }
                }
            }

            Ok(())
        }

        Ast::Function {
            name,
            parameters,
            parameter_types,
            body,
            return_type,
            attributes,
            span,
            ..
        } => {
            typechecker
                .get_mut_context()
                .set_current_function_type((return_type, *span));

            if !typechecker.get_table().constains_function(name) {
                typechecker
                    .get_mut_table()
                    .new_function(name, (return_type, parameter_types, attributes));
            }

            {
                for node in parameters.iter() {
                    let type_: &Type = node.get_any_type()?;
                    let span: Span = node.get_span();

                    if type_.is_void_type() {
                        typechecker.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0019,
                            "Void type isn't a value.".into(),
                            None,
                            span,
                        ));
                    }
                }
            }

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

            typechecker.get_mut_context().unset_current_function_type();

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
