use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::linter::Linter;
use crate::front_end::semantic::linter::marks;
use crate::front_end::types::ast::Ast;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Mut { source, value, .. } => {
            if let Ast::Reference { name, .. } = &**source {
                marks::mark_as_used(linter, name);
                marks::mark_as_mutated(linter, name);
            }

            linter.analyze_expr(source);
            linter.analyze_expr(value);
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
