use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::LLI {
            name, span, value, ..
        } => {
            linter.symbols.new_lli(name, (*span, false));
            linter.analyze_expr(value);
        }

        Ast::Write {
            source,
            write_value,
            ..
        } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_expr(expr);
            }

            linter.analyze_expr(write_value);
        }

        Ast::Address {
            source, indexes, ..
        } => {
            indexes.iter().for_each(|indexe| {
                linter.analyze_expr(indexe);
            });

            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_expr(expr);
            }
        }

        Ast::Load { source, .. } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_expr(expr);
            }
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
