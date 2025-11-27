use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::For {
            local,
            cond,
            actions,
            block,
            ..
        } => {
            analyzer.get_mut_context().increment_loop_depth();

            analyzer.analyze_stmt(local)?;
            analyzer.analyze_expr(cond)?;

            analyzer.analyze_expr(actions)?;
            analyzer.analyze_stmt(block)?;

            analyzer.get_mut_context().decrement_loop_depth();

            Ok(())
        }

        Ast::While { cond, block, .. } => {
            analyzer.get_mut_context().increment_loop_depth();

            analyzer.analyze_expr(cond)?;
            analyzer.analyze_stmt(block)?;

            analyzer.get_mut_context().decrement_loop_depth();

            Ok(())
        }

        Ast::Loop { block, .. } => {
            analyzer.get_mut_context().increment_loop_depth();

            analyzer.analyze_stmt(block)?;

            analyzer.get_mut_context().decrement_loop_depth();

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
