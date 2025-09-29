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
        Ast::If {
            condition,
            block,
            elseif,
            anyway,
            ..
        } => {
            analyzer.analyze_stmt(condition)?;

            elseif
                .iter()
                .try_for_each(|elif| analyzer.analyze_stmt(elif))?;

            if let Some(otherwise) = anyway {
                analyzer.analyze_stmt(otherwise)?;
            }

            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Elif {
            condition, block, ..
        } => {
            analyzer.analyze_stmt(condition)?;
            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Else { block, .. } => {
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
