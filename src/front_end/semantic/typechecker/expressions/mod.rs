pub mod call;
pub mod cast;
pub mod deref;
pub mod index;
pub mod indirect;
pub mod lli;
pub mod property;

use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::builtins;
use crate::front_end::semantic::typechecker::checks;
use crate::front_end::semantic::typechecker::expressions;
use crate::front_end::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::front_end::semantic::typechecker::validations;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::types::parser::stmts::types::Constructor;
use crate::front_end::typesystem::traits::TypeArrayEntensions;
use crate::front_end::typesystem::traits::TypeFixedArrayEntensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::BinaryOp {
            left,
            operator,
            right,
            span,
            ..
        } => {
            validations::binary::validate_binary(
                operator,
                left.get_value_type()?,
                right.get_value_type()?,
                *span,
            )?;

            typechecker.analyze_expr(left)?;
            typechecker.analyze_expr(right)?;

            Ok(())
        }

        Ast::UnaryOp {
            operator,
            expression,
            span,
            ..
        } => {
            validations::unary::validate_unary(operator, expression.get_value_type()?, *span)?;

            typechecker.analyze_expr(expression)?;

            Ok(())
        }

        Ast::Group { expression, .. } => {
            typechecker.analyze_expr(expression)?;
            Ok(())
        }

        Ast::FixedArray { items, kind, span } => {
            if kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    "An element is expected for inference.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_fixed_array_base_type();

            items.iter().try_for_each(|item| {
                let span: Span = item.get_span();

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(item.is_literal_value(), span);

                checks::check_types(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    metadata,
                )?;

                typechecker.analyze_expr(item)
            })?;

            Ok(())
        }

        Ast::Array {
            items, kind, span, ..
        } => {
            if kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    "Type error".into(),
                    "An element is expected for inference.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_array_base_type();

            items.iter().try_for_each(|item| {
                let span: Span = item.get_span();

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(item.is_literal_value(), span);

                checks::check_types(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    metadata,
                )?;

                typechecker.analyze_expr(item)
            })?;

            Ok(())
        }

        Ast::Index { .. } => index::validate(typechecker, node),
        Ast::Property { .. } => property::validate(typechecker, node),

        Ast::Constructor { args, .. } => {
            let args: &Constructor = args;

            args.iter().try_for_each(|arg| {
                let expr: &Ast = &arg.1;
                let span: Span = expr.get_span();

                let target_type: &Type = &arg.2;
                let from_type: &Type = expr.get_value_type()?;

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(expr.is_literal_value(), span);

                checks::check_types(target_type, from_type, Some(expr), None, metadata)?;

                typechecker.analyze_expr(expr)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call {
            name, args, span, ..
        } => {
            if let Some(metadata) = typechecker.get_symbols().get_function(name) {
                return expressions::call::validate(typechecker, *metadata, args, span);
            } else if let Some(metadata) = typechecker.get_symbols().get_intrinsic(name) {
                return expressions::call::validate(typechecker, *metadata, args, span);
            } else if let Some(metadata) = typechecker.get_symbols().get_asm_function(name) {
                return expressions::call::validate(typechecker, *metadata, args, span);
            }

            typechecker.add_error(CompilationIssue::FrontEndBug(
                "Function not found".into(),
                "Function could not be found for processing.".into(),
                *span,
                CompilationPosition::TypeChecker,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }

        ast if ast.is_lli() => expressions::lli::validate(typechecker, node),

        Ast::Indirect { .. } => expressions::indirect::validate(typechecker, node),

        Ast::Deref { .. } => expressions::deref::validate(typechecker, node),
        Ast::As { .. } => expressions::cast::validate(typechecker, node),
        Ast::Builtin { builtin, .. } => builtins::validate(typechecker, builtin),

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
        | Ast::Pass { .. }
        | Ast::DirectRef { .. } => Ok(()),

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
