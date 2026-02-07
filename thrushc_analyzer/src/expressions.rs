use thrushc_ast::{
    Ast,
    builitins::ThrushBuiltin,
    traits::{AstCodeLocation, AstGetType, AstMemoryExtensions, AstStandardExtensions},
};
use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};
use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{TypeExtensions, TypeIsExtensions},
};

use crate::Analyzer;

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
            {
                for node in items.iter() {
                    analyzer.analyze_expr(node)?;
                }
            }

            Ok(())
        }

        Ast::Array { items, .. } => {
            {
                for node in items.iter() {
                    analyzer.analyze_expr(node)?;
                }
            }

            Ok(())
        }

        Ast::Index { source, index, .. } => {
            let source_type: &Type = source.get_any_type()?;

            if source.is_reference() && !source.is_allocated() {
                analyzer.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0007,
                    "An reference with memory address was expected. Try to allocate it.".into(),
                    None,
                    source.get_span(),
                ));
            }

            if (!source.is_allocated_value()? || !source.is_reference()) && source_type.is_value() {
                analyzer.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0008,
                    format!(
                        "An value with memory address was expected, got '{}'. Try to allocate it.",
                        source_type
                    ),
                    None,
                    source.get_span(),
                ));
            }

            analyzer.analyze_expr(index)?;

            Ok(())
        }

        Ast::Property { source, .. } => {
            analyzer.analyze_expr(source)?;
            Ok(())
        }

        Ast::Constructor { data, .. } => {
            {
                for (_, node, ..) in data.iter() {
                    analyzer.analyze_expr(node)?;
                }
            }

            Ok(())
        }

        Ast::Call { args, .. } => args.iter().try_for_each(|arg| analyzer.analyze_expr(arg)),

        Ast::IndirectCall { function, args, .. } => {
            analyzer.analyze_expr(function)?;

            {
                for argument in args.iter() {
                    analyzer.analyze_expr(argument)?;
                }
            }

            Ok(())
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
        Ast::Deref { value, .. } => {
            analyzer.analyze_expr(value)?;
            Ok(())
        }
        Ast::As { from, .. } => {
            analyzer.analyze_expr(from)?;

            Ok(())
        }
        Ast::Builtin { builtin, .. } => match builtin {
            ThrushBuiltin::MemSet {
                dst,
                new_size,
                size,
                ..
            } => {
                analyzer.analyze_expr(dst)?;
                analyzer.analyze_expr(new_size)?;
                analyzer.analyze_expr(size)?;

                Ok(())
            }

            ThrushBuiltin::MemMove { dst, src, size, .. } => {
                analyzer.analyze_expr(dst)?;
                analyzer.analyze_expr(src)?;
                analyzer.analyze_expr(size)?;

                Ok(())
            }

            ThrushBuiltin::MemCpy { dst, src, size, .. } => {
                analyzer.analyze_expr(dst)?;
                analyzer.analyze_expr(src)?;
                analyzer.analyze_expr(size)?;

                Ok(())
            }

            ThrushBuiltin::Halloc { .. }
            | ThrushBuiltin::AlignOf { .. }
            | ThrushBuiltin::SizeOf { .. }
            | ThrushBuiltin::AbiSizeOf { .. }
            | ThrushBuiltin::BitSizeOf { .. }
            | ThrushBuiltin::AbiAlignOf { .. } => Ok(()),
        },

        Ast::AsmValue { .. }
        | Ast::EnumValue { .. }
        | Ast::Reference { .. }
        | Ast::Integer { .. }
        | Ast::Boolean { .. }
        | Ast::CString { .. }
        | Ast::CNString { .. }
        | Ast::Float { .. }
        | Ast::NullPtr { .. }
        | Ast::Char { .. } => Ok(()),

        _ => {
            let span: Span = node.get_span();

            analyzer.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::Analyzer,
                std::path::PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
