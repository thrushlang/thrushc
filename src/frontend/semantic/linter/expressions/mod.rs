pub mod cast;
pub mod defer;

use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::linter::{
            Linter,
            expressions::{self},
            marks::{mutable, used},
        },
        types::{ast::Ast, parser::stmts::types::Constructor},
    },
};

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
                used::mark_as_used(linter, name);

                if operator.is_minus_minus_operator() || operator.is_plus_plus_operator() {
                    mutable::mark_as_mutated(linter, name);
                }
            }

            linter.analyze_expr(expression);
        }

        Ast::AsmValue { args, .. } => {
            args.iter().for_each(|arg| {
                linter.analyze_expr(arg);
            });
        }

        Ast::Index {
            source, indexes, ..
        } => {
            linter.analyze_expr(source);

            indexes.iter().for_each(|indexe| {
                linter.analyze_expr(indexe);
            });
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

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
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
            if let Some(function) = linter.symbols.get_function_info(name) {
                function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_expr(arg);
                });

                return;
            }

            if let Some(asm_function) = linter.symbols.get_asm_function_info(name) {
                asm_function.1 = true;

                args.iter().for_each(|arg| {
                    linter.analyze_expr(arg);
                });

                return;
            }

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
                String::from("Call not caught"),
                format!("Could not get named function '{}'.", name),
                *span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Ast::Reference { name, .. } => {
            used::mark_as_used(linter, name);
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
            if let Some((enum_name, field_name)) = linter.symbols.split_enum_field_name(name) {
                if let Some(union) = linter.symbols.get_enum_info(enum_name) {
                    union.2 = true;
                }

                if let Some(enum_field) = linter.symbols.get_enum_field_info(enum_name, field_name)
                {
                    enum_field.1 = true;
                }

                linter.analyze_expr(value);

                return;
            }

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
                String::from("Enum value not caught"),
                format!("Could not get correct name of the enum field '{}'.", name),
                *span,
                CompilationPosition::Linter,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Ast::As { .. } => expressions::cast::analyze_cast(linter, expr),
        Ast::Defer { .. } => defer::analyze_deference(linter, expr),
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

            linter.add_bug(ThrushCompilerIssue::FrontEndBug(
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
