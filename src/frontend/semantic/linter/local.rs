use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::linter::Linter,
        types::ast::{Ast, metadata::local::LocalMetadata},
    },
};

pub fn analyze_local<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Local {
            name,
            value,
            span,
            metadata,
            ..
        } => {
            let metadata: &LocalMetadata = metadata;

            linter
                .symbols
                .new_local(name, (*span, false, !metadata.is_mutable()));

            linter.analyze_ast_expr(value);
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
