use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Function {
            parameters, body, ..
        } => {
            if let Some(body) = body {
                linter.symbols.declare_parameters(parameters);
                linter.analyze_stmt(body);
                linter.symbols.finish_parameters();

                linter.generate_scoped_function_warnings();
            }
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }
    }
}
