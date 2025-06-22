use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span, semantic::linter::Linter, types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn analyze_loop<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::For {
            local,
            actions,
            cond,
            block,
            ..
        } => {
            linter.analyze_stmt(local);
            linter.analyze_stmt(actions);
            linter.analyze_stmt(cond);
            linter.analyze_stmt(block);
        }

        ThrushStatement::While { cond, block, .. } => {
            linter.analyze_stmt(cond);
            linter.analyze_stmt(block);
        }

        ThrushStatement::Loop { block, .. } => {
            linter.analyze_stmt(block);
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
