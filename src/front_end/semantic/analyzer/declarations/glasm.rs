use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;
use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::GlobalAssembler { span, .. } => {
            if analyzer.get_context().has_global_assembler() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Syntax Error".into(),
                    "Global assembler is already defined before. One per file is expected. Remove one.".into(),
                    None,
                    *span,
                ));
            }

            analyzer.get_mut_context().set_has_global_assembler();

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
