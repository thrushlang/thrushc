use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::traits::TypeExtensions;
use crate::front_end::typesystem::types::Type;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Mut { source, value, .. } => {
            let source_type: &Type = source.get_value_type()?;

            if source.is_reference() && !source.is_allocated() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Invalid reference".into(),
                    "An reference with memory address was expected. Try to allocate it.".into(),
                    None,
                    source.get_span(),
                ));
            }

            if (!source.is_allocated_value()? || !source.is_reference()) && source_type.is_value() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Invalid value".into(),
                    format!(
                        "An value with memory address was expected, got '{}'. Try to allocate it.",
                        source_type
                    ),
                    None,
                    source.get_span(),
                ));
            }

            analyzer.analyze_expr(source)?;
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
