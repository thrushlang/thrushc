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
            analyzer.analyze_stmt(left)?;
            analyzer.analyze_stmt(right)?;

            Ok(())
        }

        Ast::UnaryOp { expression, .. } => {
            analyzer.analyze_stmt(expression)?;

            Ok(())
        }

        Ast::Group { expression, .. } => {
            analyzer.analyze_stmt(expression)?;

            Ok(())
        }

        Ast::FixedArray { items, .. } => {
            items
                .iter()
                .try_for_each(|item| analyzer.analyze_stmt(item))?;

            Ok(())
        }

        Ast::Array { items, .. } => {
            items
                .iter()
                .try_for_each(|item| analyzer.analyze_stmt(item))?;

            Ok(())
        }

        Ast::Index { .. } => index::validate(analyzer, node),
        Ast::Property { .. } => property::validate(analyzer, node),

        Ast::Constructor { args, .. } => {
            let args: &Constructor = args;

            args.iter().try_for_each(|arg| {
                let expr: &Ast = &arg.1;

                analyzer.analyze_stmt(expr)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call { args, .. } => args.iter().try_for_each(|arg| analyzer.analyze_stmt(arg)),

        Ast::Indirect { function, args, .. } => {
            analyzer.analyze_stmt(function)?;
            args.iter().try_for_each(|arg| analyzer.analyze_stmt(arg))
        }

        Ast::AsmValue { .. }
        | Ast::Alloc { .. }
        | Ast::EnumValue { .. }
        | Ast::Reference { .. }
        | Ast::Integer { .. }
        | Ast::Boolean { .. }
        | Ast::Str { .. }
        | Ast::Float { .. }
        | Ast::Null { .. }
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
