use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{lexer::span::Span, semantic::analyzer::Analyzer, types::ast::Ast},
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
            if let Err(error) = analyzer.analyze_stmt(local) {
                analyzer.add_error(error);
            }

            if let Err(error) = analyzer.analyze_stmt(cond) {
                analyzer.add_error(error);
            }

            if let Err(error) = analyzer.analyze_stmt(actions) {
                analyzer.add_error(error);
            }

            if let Err(error) = analyzer.analyze_stmt(block) {
                analyzer.add_error(error);
            }

            Ok(())
        }

        Ast::While { cond, block, .. } => {
            analyzer.analyze_stmt(cond)?;
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
                line!(),
            ));

            Ok(())
        }
    }
}
