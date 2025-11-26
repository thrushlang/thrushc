use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;

use std::path::PathBuf;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Const { value, .. } => {
            let span: Span = value.get_span();

            if !value.is_constant_value() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected constant value or reference constant value. Verify that it is an SSA value.".into(),
                    None,
                    span,
                ));
            }

            analyzer.analyze_expr(value)?;

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
