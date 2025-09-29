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
            value: local_value,
            metadata,
            ..
        } => {
            analyzer.symbols.new_local(name, local_type);

            let metadata: &LocalMetadata = metadata;

            if !metadata.is_undefined() {
                if let Err(type_error) = analyzer.analyze_stmt(local_value) {
                    analyzer.add_error(type_error);
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
