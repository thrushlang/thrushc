use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstConstantExtensions};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Static { value, .. } => {
            if let Some(value) = value {
                if !value.is_constant_value() {
                    analyzer.add_error(CompilationIssue::Error(
                        "Syntax error".into(),
                        "Expected constant value or reference constant value. Verify that it is an SSA value.".into(),
                        None,
                        value.get_span(),
                    ));
                }

                analyzer.analyze_expr(value)?;
            }

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
