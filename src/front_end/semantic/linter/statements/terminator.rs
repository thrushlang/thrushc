use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;

use std::path::PathBuf;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Return { expression, .. } => {
            if let Some(expr) = expression {
                linter.analyze_expr(expr);
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
