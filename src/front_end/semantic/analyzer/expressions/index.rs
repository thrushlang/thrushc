use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{
    AstCodeLocation, AstGetType, AstMemoryExtensions, AstStandardExtensions,
};
use crate::front_end::typesystem::traits::TypeExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Index { source, index, .. } => {
            let source_type: &Type = source.get_any_type()?;

            if source.is_reference() && !source.is_allocated() {
                analyzer.add_error(CompilationIssue::Error(
                    "Invalid reference".into(),
                    "An reference with memory address was expected. Try to allocate it.".into(),
                    None,
                    source.get_span(),
                ));
            }

            if (!source.is_allocated_value()? || !source.is_reference()) && source_type.is_value() {
                analyzer.add_error(CompilationIssue::Error(
                    "Invalid value".into(),
                    format!(
                        "An value with memory address was expected, got '{}'. Try to allocate it.",
                        source_type
                    ),
                    None,
                    source.get_span(),
                ));
            }

            analyzer.analyze_expr(index)?;

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
