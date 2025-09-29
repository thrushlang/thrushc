use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{lexer::span::Span, semantic::analyzer::Analyzer, types::ast::Ast},
};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Property { source, .. } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;

                analyzer.analyze_stmt(reference)?;

                return Ok(());
            }

            if let Some(expr) = &source.1 {
                analyzer.analyze_stmt(expr)?;

                return Ok(());
            }

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            analyzer.add_bug(ThrushCompilerIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Analyzer,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
