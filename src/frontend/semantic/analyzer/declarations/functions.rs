use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::span::Span, semantic::analyzer::Analyzer, types::ast::Ast},
};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::EntryPoint { body, .. } => {
            analyzer.analyze_stmt(body)?;
            Ok(())
        }

        Ast::AssemblerFunction { .. } => Ok(()),

        Ast::Function { body, .. } => {
            if let Some(body) = body {
                analyzer.analyze_stmt(body)?;
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
