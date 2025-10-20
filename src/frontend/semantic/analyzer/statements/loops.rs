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
        Ast::For {
            local,
            cond,
            actions,
            block,
            ..
        } => {
            analyzer.analyze_stmt(local)?;
            analyzer.analyze_expr(cond)?;

            analyzer.analyze_expr(actions)?;
            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        Ast::While { cond, block, .. } => {
            analyzer.analyze_expr(cond)?;
            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Loop { block, .. } => {
            analyzer.analyze_stmt(block)?;

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
