use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstGetType, AstStandardExtensions},
};
use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};

use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{
        TypeArrayEntensions, TypeFixedArrayEntensions, TypeIsExtensions, TypePointerExtensions,
    },
};

use crate::{TypeChecker, checking, metadata::TypeCheckerExpressionMetadata, operations};

mod builtins;
mod call;

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
            operations::binary::validate_binary(
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
            operations::unary::validate_unary(operator, expression.get_value_type()?, *span)?;

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
                    CompilationIssueCode::E0019,
                    "An element is expected for type inference.".into(),
                    None,
                    *span,
                ));
            }

            items.iter().try_for_each(|item| {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(item.is_literal_value());
                let item_type: &Type = item.get_value_type()?;
                let base_type: Type = kind.get_fixed_array_base_type();

                checking::check_types(
                    &base_type,
                    item_type,
                    Some(item),
                    None,
                    metadata,
                    item.get_span(),
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
                    CompilationIssueCode::E0019,
                    "An element is expected for type inference.".into(),
                    None,
                    *span,
                ));
            }

            items.iter().try_for_each(|item| {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(item.is_literal_value());
                let item_type: &Type = item.get_value_type()?;
                let base_type: Type = kind.get_array_base_type();

                checking::check_types(
                    &base_type,
                    item_type,
                    Some(item),
                    None,
                    metadata,
                    item.get_span(),
                )?;

                typechecker.analyze_expr(item)
            })?;

            Ok(())
        }

        Ast::Index { source, index, .. } => {
            let index_type: &Type = index.get_value_type()?;
            let span: Span = index.get_span();

            if !index_type.is_integer_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    format!("Expected integer value, got '{}'.", index_type),
                    None,
                    span,
                ));
            }

            typechecker.analyze_expr(index)?;
            typechecker.analyze_expr(source)?;

            Ok(())
        }
        Ast::Property { source, .. } => {
            let source_type: &Type = source.get_value_type()?;
            let source_span: Span = source.get_span();

            if !source_type.is_struct_type() && !source_type.is_ptr_struct_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    format!("A structure type was expected within a structure 'struct T' type, or raw typed pointer structure pointer 'ptr[struct T]', got '{}'.", source_type),
                    None,
                    source_span,
                ));
            }

            typechecker.analyze_expr(source)?;

            Ok(())
        }

        Ast::Constructor { args, .. } => {
            args.iter().try_for_each(|arg| {
                let expr: &Ast = &arg.1;
                let span: Span = expr.get_span();

                let target_type: &Type = &arg.2;
                let from_type: &Type = expr.get_value_type()?;

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(expr.is_literal_value());

                checking::check_types(target_type, from_type, Some(expr), None, metadata, span)?;

                typechecker.analyze_expr(expr)?;

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call {
            name, args, span, ..
        } => {
            if let Some(metadata) = typechecker.get_table().get_function(name) {
                return call::validate(typechecker, *metadata, args, span);
            } else if let Some(metadata) = typechecker.get_table().get_intrinsic(name) {
                return call::validate(typechecker, *metadata, args, span);
            } else if let Some(metadata) = typechecker.get_table().get_asm_function(name) {
                return call::validate(typechecker, *metadata, args, span);
            }

            typechecker.add_error(CompilationIssue::FrontEndBug(
                "Function not found".into(),
                "Function could not be found for processing.".into(),
                *span,
                CompilationPosition::TypeChecker,
                std::path::PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }

        Ast::Deref { value, .. } => {
            let value_type: &Type = value.get_value_type()?;

            if !value_type.is_ptr_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    format!("Expected raw typed pointer 'ptr[T]' type, raw pointer 'ptr' type for defererence, got '{}'.", value_type),
                    None,
                    value.get_span(),
                ));
            }

            typechecker.analyze_expr(value)?;

            Ok(())
        }
        Ast::As {
            from,
            cast: cast_type,
            metadata,
            span,
            ..
        } => {
            let from_type: &Type = from.get_value_type()?;

            checking::check_type_cast(cast_type, from_type, metadata, span)?;

            typechecker.analyze_expr(from)?;

            Ok(())
        }

        Ast::Builtin { builtin, .. } => builtins::validate(typechecker, builtin),

        Ast::AsmValue { .. }
        | Ast::EnumValue { .. }
        | Ast::Reference { .. }
        | Ast::Integer { .. }
        | Ast::Boolean { .. }
        | Ast::Str { .. }
        | Ast::Float { .. }
        | Ast::NullPtr { .. }
        | Ast::Char { .. }
        | Ast::DirectRef { .. } => Ok(()),

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(CompilationIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                std::path::PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
