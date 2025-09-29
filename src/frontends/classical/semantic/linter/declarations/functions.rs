use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::EntryPoint { body, .. } => {
            linter.analyze_stmt(body);
        }

        Ast::Function {
            parameters, body, ..
        } => {
            if body.is_block() {
                linter.symbols.bulk_declare_parameters(parameters);

                linter.analyze_stmt(body);
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
