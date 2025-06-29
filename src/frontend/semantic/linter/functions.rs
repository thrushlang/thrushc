use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze_function<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::EntryPoint { body, .. } => {
            linter.analyze_ast_stmt(body);
        }

        Ast::Function {
            parameters, body, ..
        } => {
            if body.is_block() {
                linter.symbols.bulk_declare_parameters(parameters);

                linter.analyze_ast_stmt(body);

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
