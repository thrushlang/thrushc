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
            linter.analyze_ast(value);
        }

        Ast::Write {
            write_to,
            write_value,
            ..
        } => {
            if let Some(any_reference) = &write_to.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast(reference);
            }

            if let Some(expr) = &write_to.1 {
                linter.analyze_ast(expr);
            }

            linter.analyze_ast(write_value);
        }

        Ast::Address {
            address_to,
            indexes,
            ..
        } => {
            indexes.iter().for_each(|indexe| {
                linter.analyze_ast(indexe);
            });

            if let Some(any_reference) = &address_to.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast(reference);
            }

            if let Some(expr) = &address_to.1 {
                linter.analyze_ast(expr);
            }
        }

        Ast::Load { value, .. } => {
            if let Some(any_reference) = &value.0 {
                let reference: &Ast = &any_reference.1;
                linter.analyze_ast(reference);
            }

            if let Some(expr) = &value.1 {
                linter.analyze_ast(expr);
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
