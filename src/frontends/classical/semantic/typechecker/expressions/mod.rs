pub mod call;
pub mod cast;
pub mod deref;
pub mod indirect;

mod index;
mod property;

use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::{
            TypeChecker, checks, expressions, metadata::TypeCheckerExprMetadata, validations,
        },
        types::{ast::Ast, parser::stmts::types::Constructor},
        typesystem::{
            traits::{TypeArrayEntensions, TypeFixedArrayEntensions},
            types::Type,
        },
    },
};

pub fn validate<'check_typeser>(
    typechecker: &mut TypeChecker<'check_typeser>,
    node: &'check_typeser Ast,
) -> Result<(), ThrushCompilerIssue> {
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

            typechecker.analyze_stmt(left)?;
            typechecker.analyze_stmt(right)?;

            Ok(())
        }

        Ast::UnaryOp {
            operator,
            expression,
            span,
            ..
        } => {
            validations::unary::validate_unary(operator, expression.get_value_type()?, *span)?;

            if operator.is_plus_plus_operator() || operator.is_minus_minus_operator() {
                if !expression.is_reference() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected a reference.".into(),
                        None,
                        *span,
                    ));
                }

                if !expression.is_mutable() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected a mutable reference.".into(),
                        None,
                        *span,
                    ));
                }
            }

            typechecker.analyze_stmt(expression)?;

            Ok(())
        }

        Ast::Group { expression, .. } => {
            typechecker.analyze_stmt(expression)?;
            Ok(())
        }

        Ast::FixedArray { items, kind, span } => {
            if kind.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "An element is expected.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_farray_base_type();

            items.iter().try_for_each(|item| {
                let span: Span = item.get_span();

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(item.is_literal(), None, span);

                checks::check_types(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    metadata,
                )?;

                typechecker.analyze_stmt(item)
            })?;

            Ok(())
        }

        Ast::Array {
            items, kind, span, ..
        } => {
            if kind.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "An element is expected.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_array_base_type();

            items.iter().try_for_each(|item| {
                let span: Span = item.get_span();

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(item.is_literal(), None, span);

                checks::check_types(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    metadata,
                )?;

                typechecker.analyze_stmt(item)
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
                    TypeCheckerExprMetadata::new(expr.is_literal(), None, span);

                checks::check_types(target_type, from_type, Some(expr), None, metadata)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call {
            name, args, span, ..
        } => {
            if let Some(metadata) = typechecker.symbols.get_function(name) {
                return expressions::call::validate(typechecker, *metadata, args, span);
            } else if let Some(metadata) = typechecker.symbols.get_asm_function(name) {
                return expressions::call::validate(typechecker, *metadata, args, span);
            }

            typechecker.add_error(ThrushCompilerIssue::FrontEndBug(
                "Function not found".into(),
                "Function could not be found for processing.".into(),
                *span,
                CompilationPosition::TypeChecker,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }

        Ast::Indirect { .. } => expressions::indirect::validate(typechecker, node),

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

            typechecker.add_bug(ThrushCompilerIssue::FrontEndBug(
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
