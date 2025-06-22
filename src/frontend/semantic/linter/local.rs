use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span, semantic::linter::Linter, types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn analyze_local<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::Local {
            name,
            value,
            span,
            is_mutable,
            ..
        } => {
            linter.symbols.new_local(name, (*span, false, !is_mutable));
            linter.analyze_stmt(value);
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
