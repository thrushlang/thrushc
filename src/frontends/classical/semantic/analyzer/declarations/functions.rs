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
        Ast::EntryPoint { body, .. } => {
            if let Err(type_error) = analyzer.analyze_stmt(body) {
                analyzer.add_error(type_error);
            }

            Ok(())
        }

        Ast::AssemblerFunction { parameters, .. } => {
            parameters.iter().try_for_each(|parameter| {
                analyzer.analyze_stmt(parameter)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Function { body, .. } => {
            if body.is_block() {
                if let Err(error) = analyzer.analyze_stmt(body) {
                    analyzer.add_error(error);
                }
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
