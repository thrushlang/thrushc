use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::{TypeChecker, bounds, call, validations},
        types::{ast::Ast, parser::stmts::types::Constructor},
        typesystem::{
            traits::{TypeMutableExtensions, TypePointerExtensions},
            types::Type,
        },
    },
};

pub fn validate_expression<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::BinaryOp {
            left,
            operator,
            right,
            span,
            ..
        } => {
            if let Err(mismatch_type_error) = validations::binary::validate_binary(
                operator,
                left.get_value_type()?,
                right.get_value_type()?,
                *span,
            ) {
                typechecker.add_error(mismatch_type_error);
            }

            if let Err(type_error) = typechecker.analyze_ast(left) {
                typechecker.add_error(type_error);
            }

            if let Err(type_error) = typechecker.analyze_ast(right) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        Ast::UnaryOp {
            operator,
            expression,
            span,
            ..
        } => {
            if let Err(mismatch_type_error) =
                validations::unary::validate_unary(operator, expression.get_value_type()?, *span)
            {
                typechecker.add_error(mismatch_type_error);
            }

            if let TokenType::PlusPlus | TokenType::MinusMinus = *operator {
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

            if let Err(type_error) = typechecker.analyze_ast(expression) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        Ast::Group { expression, .. } => {
            if let Err(type_error) = typechecker.analyze_ast(expression) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        Ast::Mut {
            source,
            value,
            span,
            ..
        } => {
            let value_type: &Type = value.get_value_type()?;

            if let (Some(any_reference), None) = source {
                let reference: &Ast = &any_reference.1;
                let reference_type: &Type = reference.get_value_type()?;

                if !reference.is_allocated_reference() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected mutable, or pointer reference.".into(),
                        None,
                        *span,
                    ));
                }

                if !reference.is_mutable() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The reference must be mutable.".into(),
                        None,
                        reference.get_span(),
                    ));
                }

                if reference_type.is_mut_type() {
                    let reference_type: Type = reference_type.defer_mut_all();

                    if let Err(error) = bounds::checking::check(
                        &reference_type,
                        value_type,
                        Some(value),
                        None,
                        None,
                        span,
                    ) {
                        typechecker.add_error(error);
                    }
                } else if let Err(error) = bounds::checking::check(
                    reference_type,
                    value_type,
                    Some(value),
                    None,
                    None,
                    span,
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_ast(value)?;

                return Ok(());
            }

            if let (None, Some(source)) = source {
                let source_type: &Type = source.get_value_type()?;

                if !source_type.is_ptr_type() && !source_type.is_mut_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr[T]', 'ptr', or 'mut T' type.".into(),
                        None,
                        *span,
                    ));
                }

                if !source.is_mutable() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "The reference must be mutable.".into(),
                        None,
                        source.get_span(),
                    ));
                }

                if source_type.is_mut_type() {
                    let source_type: Type = source_type.defer_mut_all();

                    if let Err(error) = bounds::checking::check(
                        &source_type,
                        value_type,
                        Some(value),
                        None,
                        None,
                        span,
                    ) {
                        typechecker.add_error(error);
                    }
                } else if let Err(error) =
                    bounds::checking::check(source_type, value_type, Some(value), None, None, span)
                {
                    typechecker.add_error(error);
                }

                typechecker.analyze_ast(value)?;

                return Ok(());
            }

            typechecker.errors.push(ThrushCompilerIssue::Bug(
                String::from("Non-trapped mutable expression."),
                String::from("The mutable expression could not be caught for processing."),
                *span,
                CompilationPosition::TypeChecker,
                line!(),
            ));

            Ok(())
        }

        Ast::FixedArray { items, kind, span } => {
            if kind.is_void_type() {
                return Err(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "An element is expected.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_fixed_array_base_type();

            items.iter().try_for_each(|item| {
                if let Err(error) = bounds::checking::check(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    None,
                    &item.get_span(),
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_ast(item)
            })?;

            Ok(())
        }

        Ast::Array {
            items, kind, span, ..
        } => {
            if kind.is_void_type() {
                return Err(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "An element is expected.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &Type = kind.get_array_base_type();

            items.iter().try_for_each(|item| {
                if let Err(error) = bounds::checking::check(
                    array_type,
                    item.get_value_type()?,
                    Some(item),
                    None,
                    None,
                    &item.get_span(),
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_ast(item)
            })?;

            Ok(())
        }

        Ast::Index {
            index_to,
            indexes,
            span,
            ..
        } => {
            if let Some(any_reference) = &index_to.0 {
                let reference: &Ast = &any_reference.1;

                if !reference.is_allocated_reference() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected a allocated value.".into(),
                        None,
                        *span,
                    ));
                }

                let reference_type: &Type = reference.get_value_type()?;

                if reference_type.is_ptr_type() && !reference_type.is_typed_ptr_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr[T].".into(),
                        None,
                        *span,
                    ));
                } else if reference_type.is_ptr_type()
                    && reference_type.is_typed_ptr_type()
                    && reference_type.is_all_ptr_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        *span,
                    ));
                } else if !reference_type.is_mut_array_type()
                    && !reference_type.is_mut_fixed_array_type()
                    && !reference_type.is_array_type()
                    && !reference_type.is_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected deep type, array, or fixed array.".into(),
                        None,
                        *span,
                    ));
                }
            }

            if let Some(expr) = &index_to.1 {
                let expr_type: &Type = expr.get_any_type()?;

                if expr_type.is_ptr_type() && !expr_type.is_typed_ptr_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr[T].".into(),
                        None,
                        *span,
                    ));
                } else if expr_type.is_ptr_type()
                    && expr_type.is_typed_ptr_type()
                    && expr_type.is_all_ptr_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        *span,
                    ));
                } else if !expr_type.is_mut_array_type()
                    && !expr_type.is_mut_fixed_array_type()
                    && !expr_type.is_array_type()
                    && !expr_type.is_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected deep type, array, or fixed array.".into(),
                        None,
                        *span,
                    ));
                }
            }

            indexes.iter().try_for_each(|indexe| {
                if !indexe.is_unsigned_integer()? {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected any unsigned integer value.".into(),
                        None,
                        *span,
                    ));
                }

                typechecker.analyze_ast(indexe)
            })?;

            Ok(())
        }

        Ast::Property { reference, .. } => {
            let reference_type: &Type = reference.get_value_type()?;
            let reference_span: Span = reference.get_span();

            if !reference_type.is_struct_type()
                && !reference_type.is_mut_struct_type()
                && !reference_type.is_ptr_struct_type()
            {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected reference to a struct type.".into(),
                    None,
                    reference_span,
                ));
            }

            typechecker.analyze_ast(reference)?;

            Ok(())
        }

        Ast::Constructor { args, .. } => {
            let args: &Constructor = args;

            args.iter().try_for_each(|arg| {
                let expression: &Ast = &arg.1;
                let expression_span: Span = expression.get_span();

                let target_type: &Type = &arg.2;
                let from_type: &Type = expression.get_value_type()?;

                if let Err(error) = bounds::checking::check(
                    target_type,
                    from_type,
                    Some(expression),
                    None,
                    None,
                    &expression_span,
                ) {
                    typechecker.add_error(error);
                }

                Ok(())
            })?;

            Ok(())
        }

        Ast::Call {
            name, args, span, ..
        } => {
            if let Some(metadata) = typechecker.symbols.get_function(name) {
                return call::validate_call(typechecker, *metadata, args, span);
            }

            if let Some(metadata) = typechecker.symbols.get_asm_function(name) {
                return call::validate_call(typechecker, *metadata, args, span);
            }

            Ok(())
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

            typechecker.add_bug(ThrushCompilerIssue::Bug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                line!(),
            ));

            Ok(())
        }
    }
}
