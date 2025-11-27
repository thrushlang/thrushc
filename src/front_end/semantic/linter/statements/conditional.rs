use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;

use std::path::PathBuf;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::If {
            condition,
            block,
            elseif,
            anyway,
            ..
        } => {
            linter.analyze_expr(condition);
            linter.analyze_stmt(block);

            elseif.iter().for_each(|elif| {
                linter.analyze_stmt(elif);
            });

            if let Some(otherwise) = anyway {
                linter.analyze_stmt(otherwise);
            }
        }

        Ast::Elif {
            condition, block, ..
        } => {
            linter.analyze_expr(condition);
            linter.analyze_stmt(block);
        }

        Ast::Else { block, .. } => {
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
