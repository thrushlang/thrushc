use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::analyzer::Analyzer,
        types::ast::{Ast, metadata::local::LocalMetadata},
    },
};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Local {
            name,
            kind: local_type,
            value,
            metadata,
            ..
        } => {
            analyzer.symbols.new_local(name, local_type);

            if let Some(value) = value {
                let metadata: &LocalMetadata = metadata;

                if !metadata.is_undefined() {
                    analyzer.analyze_stmt(value)?;
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
