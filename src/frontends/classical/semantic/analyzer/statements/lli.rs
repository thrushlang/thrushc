use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span, semantic::analyzer::Analyzer, types::ast::Ast, typesystem::types::Type,
    },
};

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
            analyzer.analyze_stmt(expr)?;

            Ok(())
        }

        Ast::Load { source, .. } => {
            if let Some(left) = &source.0 {
                let reference: &Ast = &left.1;
                analyzer.analyze_stmt(reference)?;
            }

            if let Some(expr) = &source.1 {
                analyzer.analyze_stmt(expr)?;
            }

            Ok(())
        }

        Ast::Address {
            source, indexes, ..
        } => {
            if let Some(reference_any) = &source.0 {
                let reference: &Ast = &reference_any.1;
                let reference_type: &Type = reference.get_value_type()?;
                let span: Span = reference.get_span();

                if reference_type.is_address_type() {
                    analyzer.add_warning(ThrushCompilerIssue::Warning(
                        "Undefined behavior".into(), 
                        "*Maybe* this value at runtime causes undefined behavior because it is anything at runtime, and memory calculation needs valid pointers or deep types.".into(), 
                       span
                    ));
                }

                analyzer.analyze_stmt(reference)?;
            }

            if let Some(expr) = &source.1 {
                let expr_type: &Type = expr.get_value_type()?;
                let span: Span = expr.get_span();

                if expr_type.is_address_type() {
                    analyzer.add_warning(ThrushCompilerIssue::Warning(
                        "Undefined behavior".into(), 
                        "*Maybe* this value at runtime causes undefined behavior because it is anything at runtime, and memory calculation needs valid pointers or deep types.".into(), 
                        span
                    ));
                }

                analyzer.analyze_stmt(expr)?;
            }

            indexes.iter().try_for_each(|indexe| {
                analyzer.analyze_stmt(indexe)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Write { source, .. } => {
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;

                analyzer.analyze_stmt(reference)?;
            }

            if let Some(expr) = &source.1 {
                analyzer.analyze_stmt(expr)?;
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
