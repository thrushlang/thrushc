use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Static {
            name,
            value,
            metadata,
            span,
            ..
        } => {
            linter
                .symbols
                .new_local_static(name, (*span, false, !metadata.is_mutable()));

            linter.analyze_expr(value);
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                line!(),
            ));
        }
    }
}
