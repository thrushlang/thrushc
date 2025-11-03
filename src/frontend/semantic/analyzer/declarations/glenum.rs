use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::analyzer::Analyzer;
use crate::frontend::types::ast::Ast;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Enum { fields, .. } => {
            fields.iter().try_for_each(|field| {
                let expr: &Ast = &field.2;
                let span: Span = expr.get_span();

                if !expr.is_constant_value() {
                    analyzer.add_error(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected compile-time known value.".into(),
                        None,
                        span,
                    ));
                }

                analyzer.analyze_expr(expr)?;

                Ok(())
            })?;

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
