use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::linter::Linter;
use crate::frontend::types::ast::Ast;

pub fn analyze_cast<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::As { from, .. } => {
            linter.analyze_expr(from);
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
