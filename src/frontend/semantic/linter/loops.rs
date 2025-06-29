use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze_loop<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::For {
            local,
            actions,
            cond,
            block,
            ..
        } => {
            linter.analyze_ast_stmt(local);
            linter.analyze_ast_expr(actions);
            linter.analyze_ast_expr(cond);
            linter.analyze_ast_expr(block);
        }

        Ast::While { cond, block, .. } => {
            linter.analyze_ast_expr(cond);
            linter.analyze_ast_stmt(block);
        }

        Ast::Loop { block, .. } => {
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
