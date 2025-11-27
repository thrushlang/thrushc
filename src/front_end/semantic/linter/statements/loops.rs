use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::For {
            local,
            actions,
            cond,
            block,
            ..
        } => {
            linter.analyze_stmt(local);
            linter.analyze_expr(actions);
            linter.analyze_expr(cond);
            linter.analyze_stmt(block);
        }

        Ast::While { cond, block, .. } => {
            linter.analyze_expr(cond);
            linter.analyze_stmt(block);
        }

        Ast::Loop { block, .. } => {
            linter.analyze_stmt(block);
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
