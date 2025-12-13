use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::linter::Linter;
use crate::front_end::semantic::linter::builtins;
use crate::front_end::semantic::linter::expressions;
use crate::front_end::semantic::linter::marks;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::types::parser::stmts::types::Constructor;

use std::path::PathBuf;

pub mod cast;
pub mod deref;
pub mod lli;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, expr: &'linter Ast) {
    match expr {
        Ast::Group { expression, .. } => {
            linter.analyze_expr(expression);
        }

        Ast::BinaryOp { left, right, .. } => {
            linter.analyze_expr(left);
            linter.analyze_expr(right);
        }

        Ast::UnaryOp {
            operator,
            expression,
            ..
        } => {
            if let Ast::Reference { name, .. } = &**expression {
                marks::mark_as_used(linter, name);

                if operator.is_minus_minus_operator() || operator.is_plus_plus_operator() {
                    marks::mark_as_mutated(linter, name);
                }
            }

            linter.analyze_expr(expression);
        }

        Ast::AsmValue { args, .. } => {
            args.iter().for_each(|arg| {
                linter.analyze_expr(arg);
            });
        }

        Ast::Index { source, index, .. } => {
            linter.analyze_expr(source);
            linter.analyze_expr(index);
        }

        Ast::Property { source, .. } => {
            linter.analyze_expr(source);
        }

        Ast::Constructor {
            name, args, span, ..
        } => {
            let args: &Constructor = args;

            args.iter().for_each(|arg| {
                let stmt: &Ast = &arg.1;
                linter.analyze_expr(stmt);
            });

            if let Some(structure) = linter.symbols.get_struct_info(name) {
                structure.2 = true;
                return;
            }

            linter.add_bug(CompilationIssue::FrontEndBug(
                String::from("Structure not caught"),
                format!("Could not get named struct with name '{}'.", name),
                *span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Ast::Indirect { function, args, .. } => {
            linter.analyze_expr(function);

            args.iter().for_each(|expr| {
                linter.analyze_expr(expr);
            });
        }

        Ast::Call {
            name, span, args, ..
        } => {
            if let Some(function) = linter.get_mut_symbols().get_function_info(name) {
                function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_expr(arg);
                });

                return;
            }

            if let Some(asm_function) = linter.get_mut_symbols().get_asm_function_info(name) {
                asm_function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_expr(arg);
                });

                return;
            }

            if let Some(intrinsic) = linter.get_mut_symbols().get_intrinsic_info(name) {
                intrinsic.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_expr(arg);
                });

                return;
            }

            linter.add_bug(CompilationIssue::FrontEndBug(
                String::from("Call not caught"),
                format!("Could not get named function '{}'.", name),
                *span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Ast::Reference { name, .. } => {
            marks::mark_as_used(linter, name);
        }

        Ast::FixedArray { items, .. } | Ast::Array { items, .. } => {
            items.iter().for_each(|item| {
                linter.analyze_expr(item);
            });
        }

        Ast::Mut { source, .. } => {
            linter.analyze_expr(source);
        }

        Ast::EnumValue {
            name, value, span, ..
        } => {
            if let Some((enum_name, field_name)) =
                linter.get_mut_symbols().split_enum_field_name(name)
            {
                if let Some(union) = linter.get_mut_symbols().get_enum_info(enum_name) {
                    union.2 = true;
                }

                if let Some(enum_field) = linter
                    .get_mut_symbols()
                    .get_enum_field_info(enum_name, field_name)
                {
                    enum_field.1 = true;
                }

                linter.analyze_expr(value);

                return;
            }

            linter.add_bug(CompilationIssue::FrontEndBug(
                String::from("Enum value not caught"),
                format!("Could not get correct name of the enum field '{}'.", name),
                *span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        ast if ast.is_lli() => {
            expressions::lli::analyze(linter, expr);
        }

        Ast::Builtin { builtin, .. } => builtins::analyze(linter, builtin),
        Ast::As { .. } => expressions::cast::analyze(linter, expr),
        Ast::Deref { .. } => deref::analyze(linter, expr),
        Ast::DirectRef { expr, .. } => {
            linter.analyze_expr(expr);
        }

        Ast::Alloc { .. }
        | Ast::Integer { .. }
        | Ast::Boolean { .. }
        | Ast::Str { .. }
        | Ast::Float { .. }
        | Ast::NullPtr { .. }
        | Ast::Char { .. }
        | Ast::Pass { .. } => (),

        _ => {
            let span: Span = expr.get_span();

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
