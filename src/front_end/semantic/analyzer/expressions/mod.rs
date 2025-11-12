pub mod cast;
pub mod deref;
pub mod index;
pub mod lli;
pub mod property;

use std::path::PathBuf;

use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::analyzer::{Analyzer, builtins};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::types::Constructor;
use crate::front_end::typesystem::types::Type;

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
            let expr_type: &Type = expr.get_value_type()?;

            if expr.is_reference() && !expr.is_allocated() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Invalid reference".into(),
                    "An reference with direction was expected.".into(),
                    None,
                    *span,
                ));
            } else if !expr.is_reference() && !expr_type.is_ptr_like_type() {
                analyzer.add_error(ThrushCompilerIssue::Error(
                    "Invalid reference".into(),
                    "An value with direction was expected.".into(),
                    None,
                    *span,
                ));
            }

            analyzer.analyze_expr(expr)?;

            Ok(())
        }

        ast if ast.is_lli() => lli::validate(analyzer, node),

        Ast::Defer { .. } => deref::validate(analyzer, node),
        Ast::As { .. } => cast::validate(analyzer, node),
        Ast::Builtin { builtin, .. } => builtins::validate(analyzer, builtin),

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
