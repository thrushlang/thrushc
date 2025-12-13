use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::linter::Linter;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;

use std::path::PathBuf;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::LLI {
            name, span, expr, ..
        } => {
            linter.symbols.new_lli(name, (*span, false));
            linter.analyze_expr(expr);
        }

        Ast::Write {
            source,
            write_value,
            ..
        } => {
            linter.analyze_expr(source);
            linter.analyze_expr(write_value);
        }

        Ast::Address {
            source, indexes, ..
        } => {
            linter.analyze_expr(source);

            indexes.iter().for_each(|indexe| {
                linter.analyze_expr(indexe);
            });
        }

        Ast::Load { source, .. } => {
            linter.analyze_expr(source);
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }
    }
}
