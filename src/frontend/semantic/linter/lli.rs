use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::linter::Linter, types::ast::Ast},
};

pub fn analyze_lli<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::LLI {
            name, span, value, ..
        } => {
            linter.symbols.new_lli(name, (*span, false));
            linter.analyze_ast_expr(value);
        }

        Ast::Write {
            source,
            write_value,
            ..
        } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_ast_expr(expr);
            }

            linter.analyze_ast_expr(write_value);
        }

        Ast::Address {
            source, indexes, ..
        } => {
            indexes.iter().for_each(|indexe| {
                linter.analyze_ast_expr(indexe);
            });

            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_ast_expr(expr);
            }
        }

        Ast::Load { source, .. } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast_expr(reference);
            }

            if let Some(expr) = &source.1 {
                linter.analyze_ast_expr(expr);
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
