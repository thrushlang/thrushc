use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::EntryPoint { body, .. } => {
            linter.analyze_stmt(body);
        }

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

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
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
