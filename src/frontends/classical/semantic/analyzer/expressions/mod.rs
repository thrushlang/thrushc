pub mod cast;
pub mod deref;

mod index;
mod property;

use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::analyzer::Analyzer,
        types::{ast::Ast, parser::stmts::types::Constructor},
    },
};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::BinaryOp { left, right, .. } => {
            analyzer.analyze_expr(left)?;
            analyzer.analyze_expr(right)?;

            Ok(())
        }

        Ast::UnaryOp { expression, .. } => {
            analyzer.analyze_expr(expression)?;

            Ok(())
        }

        Ast::Group { expression, .. } => {
            analyzer.analyze_expr(expression)?;

            Ok(())
        }

        Ast::FixedArray { items, .. } => {
            items
                .iter()
                .try_for_each(|item| analyzer.analyze_expr(item))?;

            Ok(())
        }

        Ast::Array { items, .. } => {
            items
                .iter()
                .try_for_each(|item| analyzer.analyze_expr(item))?;

            Ok(())
        }

        Ast::Index { .. } => index::validate(analyzer, node),
        Ast::Property { .. } => property::validate(analyzer, node),

        Ast::Constructor { args, .. } => {
            let args: &Constructor = args;

            args.iter().try_for_each(|arg| {
                let expr: &Ast = &arg.1;

                analyzer.analyze_expr(expr)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call { args, .. } => args.iter().try_for_each(|arg| analyzer.analyze_expr(arg)),

        Ast::Indirect { function, args, .. } => {
            analyzer.analyze_expr(function)?;
            args.iter().try_for_each(|arg| analyzer.analyze_expr(arg))
        }

        Ast::DirectRef { expr, span, .. } => {
            if !expr.is_reference() && !expr.is_allocated() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Expected allocated value".into(),
                    "Expected allocated value reference or value type with raw typed pointer 'ptr[T]', raw pointer 'ptr', array type 'array[T]', memory address 'addr', or function reference pointer 'Fn[..] -> T'."
                        .into(),
                    None,
                    *span,
                ));
            }

            analyzer.analyze_expr(expr)?;

            Ok(())
        }

        Ast::AsmValue { .. }
        | Ast::Alloc { .. }
        | Ast::EnumValue { .. }
        | Ast::Reference { .. }
        | Ast::Integer { .. }
        | Ast::Boolean { .. }
        | Ast::Str { .. }
        | Ast::Float { .. }
        | Ast::NullPtr { .. }
        | Ast::Char { .. }
        | Ast::Pass { .. } => Ok(()),

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
