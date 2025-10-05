use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::linter::{
            Linter,
            marks::{mutable, used},
        },
        types::ast::Ast,
    },
};

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Mut { source, value, .. } => {
            if let Ast::Reference { name, .. } = &**source {
                used::mark_as_used(linter, name);
                mutable::mark_as_mutated(linter, name);
            }

            linter.analyze_expr(source);
            linter.analyze_expr(value);
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
