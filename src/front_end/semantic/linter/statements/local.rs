use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::linter::Linter;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::local::LocalMetadata;

use std::path::PathBuf;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, node: &'linter Ast) {
    match node {
        Ast::Local {
            name,
            value,
            span,
            metadata,
            ..
        } => {
            let metadata: &LocalMetadata = metadata;

            linter
                .symbols
                .new_local(name, (*span, false, !metadata.is_mutable()));

            if let Some(value) = value {
                linter.analyze_expr(value);
            }
        }

        _ => {
            let span: Span = node.get_span();

            linter.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }
    }
}
