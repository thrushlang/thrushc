use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstGetType, AstStandardExtensions},
};
use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};

use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{
        TypeArrayEntensions, TypeCodeLocation, TypeFixedArrayEntensions, TypeIsExtensions,
        TypePointerExtensions, VoidTypeExtensions,
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
            kind,
            span,
            ..
        } => {
            let left_type: &Type = left.get_value_type()?;
            let right_type: &Type = right.get_value_type()?;

            operations::binary::validate_binary(operator, left_type, right_type, *span)?;

            typechecker.analyze_expr(left)?;
            typechecker.analyze_expr(right)?;

            if left_type.contains_void_type()
                || left_type.is_void_type()
                || right_type.contains_void_type()
                || right_type.is_void_type()
            {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    left_type.get_span(),
                ));
            }

            if kind.contains_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }

        Ast::UnaryOp {
            operator,
            expression,
            kind,
            span,
            ..
        } => {
            operations::unary::validate_unary(operator, expression.get_value_type()?, *span)?;

            typechecker.analyze_expr(expression)?;

            let expr_type: &Type = expression.get_value_type()?;

            if expr_type.contains_void_type() || expr_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    expr_type.get_span(),
                ));
            }

            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }

        Ast::Group {
            expression, kind, ..
        } => {
            typechecker.analyze_expr(expression)?;

            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

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
            } else if kind.contains_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
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
            } else if kind.contains_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            items.iter().try_for_each(|item| {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(item.is_literal_value());
                let item_type: &Type = item.get_value_type()?;
                let base_type: Type = kind.get_array_base_type();

                if item_type.contains_void_type() || item_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        item_type.get_span(),
                    ));
                }

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
            let source_type: &Type = source.get_value_type()?;
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

            if index_type.contains_void_type()
                || index_type.is_void_type()
                || source_type.contains_void_type()
                || source_type.is_void_type()
            {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    index_type.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Property { source, data, .. } => {
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

            if source_type.contains_void_type() || source_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    source_type.get_span(),
                ));
            }

            for (ty, (subtype, ..)) in data.iter() {
                if ty.contains_void_type() || ty.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        ty.get_span(),
                    ));
                }

                if subtype.contains_void_type() || subtype.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        subtype.get_span(),
                    ));
                }
            }

            Ok(())
        }

        Ast::Constructor { data, .. } => {
            for (_, expr, target_type, _) in data.iter() {
                let span: Span = expr.get_span();
                let from_type: &Type = expr.get_value_type()?;

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(expr.is_literal_value());

                checking::check_types(target_type, from_type, Some(expr), None, metadata, span)?;

                typechecker.analyze_expr(expr)?;

                if target_type.contains_void_type() || target_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        target_type.get_span(),
                    ));
                }

                if from_type.contains_void_type() || from_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        from_type.get_span(),
                    ));
                }
            }

            Ok(())
        }

        Ast::Call {
            name, args, span, ..
        } => {
            if let Some(metadata) = typechecker.get_table().get_function(name) {
                return call::validate(typechecker, *metadata, args, span);
            }

            if let Some(metadata) = typechecker.get_table().get_intrinsic(name) {
                return call::validate(typechecker, *metadata, args, span);
            }

            if let Some(metadata) = typechecker.get_table().get_asm_function(name) {
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

        Ast::IndirectCall {
            function,
            function_type,
            args,
            ..
        } => {
            if !function_type.is_function_reference_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "Expected  valid function reference for call anonymously.".into(),
                    None,
                    function.get_span(),
                ));
            }

            {
                for argument in args.iter() {
                    typechecker.analyze_expr(argument)?;
                }
            }

            Ok(())
        }

        Ast::Deref { value, kind, .. } => {
            let value_type: &Type = value.get_value_type()?;

            if !value_type.is_ptr_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0008,
                    format!(
                        "An value with memory address was expected, got '{}'. Try to allocate it.",
                        value
                    ),
                    None,
                    value.get_span(),
                ));
            }

            typechecker.analyze_expr(value)?;

            if value_type.contains_void_type() || value_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    value_type.get_span(),
                ));
            }

            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

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

            if cast_type.contains_void_type() || cast_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    cast_type.get_span(),
                ));
            }

            if from_type.contains_void_type() || from_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    from_type.get_span(),
                ));
            }

            Ok(())
        }

        Ast::Builtin { builtin, .. } => builtins::validate(typechecker, builtin),

        Ast::AsmValue { args, kind, .. } => {
            for node in args.iter() {
                let node_type: &Type = node.get_value_type()?;

                if node_type.contains_void_type() || node_type.is_void_type() {
                    typechecker.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        node_type.get_span(),
                    ));
                }
            }

            if kind.contains_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }

        Ast::EnumValue { value, kind, .. } => {
            let node_type: &Type = value.get_value_type()?;

            if node_type.contains_void_type() || node_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    node_type.get_span(),
                ));
            }

            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Reference { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Integer { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Boolean { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::CString { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::CNString { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Float { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::NullPtr { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::Char { kind, .. } => {
            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }
        Ast::DirectRef { expr, kind, .. } => {
            let expr_type: &Type = expr.get_value_type()?;

            if expr_type.contains_void_type() || expr_type.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    expr_type.get_span(),
                ));
            }

            if kind.contains_void_type() || kind.is_void_type() {
                typechecker.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0019,
                    "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                    None,
                    kind.get_span(),
                ));
            }

            Ok(())
        }

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
