use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span, semantic::linter::Linter, types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn analyze_enum<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::Enum { fields, .. } => {
            fields.iter().for_each(|field| {
                let expr: &ThrushStatement = &field.1;
                linter.analyze_stmt(expr);
            });
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
