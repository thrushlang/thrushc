use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::linter::Linter,
        types::{ parser::stmts::stmt::ThrushStatement},
    },
};

pub fn analyze_function<'linter>(linter: &mut Linter<'linter>, node: &'linter ThrushStatement) {
    match node {
        ThrushStatement::EntryPoint { body, .. } => {
            linter.analyze_stmt(body);
        }

        ThrushStatement::Function {
            parameters, body, ..
        } => {
            if body.is_block() {
                linter.symbols.bulk_declare_parameters(parameters);

                linter.analyze_stmt(body);

                linter.symbols.destroy_all_parameters();
            }
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
