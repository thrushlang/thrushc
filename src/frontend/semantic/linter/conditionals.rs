use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span, semantic::linter::Linter, types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn analyze_conditional<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::If {
            cond,
            block,
            elfs,
            otherwise,
            ..
        } => {
            linter.analyze_stmt(cond);
            linter.analyze_stmt(block);

            elfs.iter().for_each(|elif| {
                linter.analyze_stmt(elif);
            });

            if let Some(otherwise) = otherwise {
                linter.analyze_stmt(otherwise);
            }
        }

        ThrushStatement::Elif { cond, block, .. } => {
            linter.analyze_stmt(cond);
            linter.analyze_stmt(block);
        }

        ThrushStatement::Else { block, .. } => {
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
