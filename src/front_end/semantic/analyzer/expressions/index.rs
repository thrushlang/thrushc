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
        Ast::Index {
            source,
            indexes,
            span,
            ..
        } => {
            let source_type: &Type = source.get_any_type()?;

            if indexes.len() > 1 {
                let subtype: &Type = source_type.get_type_with_depth(1);

                if subtype.is_ptr_like_type() || source_type.is_ptr_like_type() {
                    analyzer.add_error(ThrushCompilerIssue::Error(
                        "Invalid consecutive indexing".into(),
                        "Consecutive indexing isn't allowed while it's using a pointer-to-pointer type."
                            .into(),
                        None,
                        *span,
                    ));
                }
            }

            indexes
                .iter()
                .try_for_each(|indexe| analyzer.analyze_expr(indexe))?;

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
