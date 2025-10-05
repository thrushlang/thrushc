use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::analyzer::Analyzer,
        types::ast::Ast,
        typesystem::{
            traits::{LLVMTypeExtensions, TypeExtensions},
            types::Type,
        },
    },
};

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

            if source_type.llvm_is_ptr_type() {
                let subtype: &Type = source_type.get_type_with_depth(1);

                if subtype.llvm_is_ptr_type() && indexes.len() > 1 {
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
                .try_for_each(|indexe| analyzer.analyze_stmt(indexe))?;

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
