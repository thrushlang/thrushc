use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::LLI {
            name,
            kind: lli_type,
            expr,
            span,
            ..
        } => {
            analyzer.symbols.new_lli(name, (lli_type, *span));
            analyzer.analyze_expr(expr)?;

            Ok(())
        }

        Ast::Load { source, .. } => {
            analyzer.analyze_expr(source)?;

            Ok(())
        }

        Ast::Address {
            source, indexes, ..
        } => {
            let source_type: &Type = source.get_value_type()?;
            let span: Span = source.get_span();

            if source_type.is_address_type() {
                analyzer.add_warning(ThrushCompilerIssue::Warning(
                    "Undefined behavior".into(), 
                    "*Maybe* this value at runtime causes undefined behavior because it is anything at runtime, and memory calculation needs valid pointers or deep types.".into(), 
                    span
                ));
            }

            analyzer.analyze_expr(source)?;

            indexes.iter().try_for_each(|indexe| {
                analyzer.analyze_expr(indexe)?;
                Ok(())
            })?;

            Ok(())
        }

        Ast::Write { source, .. } => {
            analyzer.analyze_expr(source)?;
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
