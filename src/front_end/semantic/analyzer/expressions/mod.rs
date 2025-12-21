use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::semantic::analyzer::builtins;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstMemoryExtensions;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::types::parser::stmts::types::Constructor;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub mod cast;
pub mod deref;
pub mod index;
pub mod lli;
pub mod property;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    node: &'analyzer Ast,
) -> Result<(), CompilationIssue> {
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
                analyzer.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0007,
                    "An reference with memory address was expected. Try to allocate it.".into(),
                    None,
                    *span,
                ));
            } else if !expr.is_reference() && !expr_type.is_ptr_like_type() {
                analyzer.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0008,
                    format!(
                        "An value with memory address was expected, got '{}'. Try to allocate it.",
                        expr_type
                    ),
                    None,
                    *span,
                ));
            }

            analyzer.analyze_expr(expr)?;

            Ok(())
        }

        ast if ast.is_lli() => lli::validate(analyzer, node),

        Ast::Deref { .. } => deref::validate(analyzer, node),
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

            analyzer.add_bug(CompilationIssue::FrontEndBug(
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
