use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze_conditional<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::If {
            cond,
            block,
            elfs,
            otherwise,
            ..
        } => {
            linter.analyze_ast_expr(cond);
            linter.analyze_ast_expr(block);

            elfs.iter().for_each(|elif| {
                linter.analyze_ast_expr(elif);
            });

            if let Some(otherwise) = otherwise {
                linter.analyze_ast_expr(otherwise);
            }
        }

        Ast::Elif { cond, block, .. } => {
            linter.analyze_ast_expr(cond);
            linter.analyze_ast_stmt(block);
        }

        Ast::Else { block, .. } => {
            linter.analyze_ast_stmt(block);
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(ThrushCompilerIssue::Bug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                line!(),
            ));
        }
    }
}
