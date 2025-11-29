use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::If {
            condition,
            block,
            elseif,
            anyway,
            ..
        } => {
            analyzer.analyze_expr(condition)?;

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
            analyzer.analyze_expr(condition)?;
            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Else { block, .. } => {
            analyzer.analyze_stmt(block)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            analyzer.add_bug(CompilationIssue::FrontEndBug(
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
