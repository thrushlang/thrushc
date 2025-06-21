use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::{TypeChecker, call, exprvalidations},
        types::{
            lexer::{
                ThrushType,
                traits::{ThrushTypeMutableExtensions, ThrushTypePointerExtensions},
            },
            parser::stmts::stmt::ThrushStatement,
        },
    },
};

pub fn validate_expression<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::BinaryOp {
            left,
            operator,
            right,
            span,
            ..
        } => {
            if let Err(mismatch_type_error) = exprvalidations::binary::validate_binary(
                operator,
                left.get_value_type()?,
                right.get_value_type()?,
                *span,
            ) {
                typechecker.add_error(mismatch_type_error);
            }

            if let Err(type_error) = typechecker.analyze_stmt(left) {
                typechecker.add_error(type_error);
            }

            if let Err(type_error) = typechecker.analyze_stmt(right) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        ThrushStatement::UnaryOp {
            operator,
            expression,
            span,
            ..
        } => {
            if let Err(mismatch_type_error) = exprvalidations::unary::validate_unary(
                operator,
                expression.get_value_type()?,
                *span,
            ) {
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

            if let Err(type_error) = typechecker.analyze_stmt(expression) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        ThrushStatement::Group { expression, .. } => {
            if let Err(type_error) = typechecker.analyze_stmt(expression) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        ThrushStatement::Mut {
            source,
            value,
            span,
            ..
        } => {
            if let (Some(any_reference), None) = source {
                let reference: &ThrushStatement = &any_reference.1;
                let reference_type: &ThrushType = reference.get_value_type()?;

                if !reference_type.is_ptr_type() && !reference_type.is_mut_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'mut T' type.".into(),
                        None,
                        *span,
                    ));
                }

                if !reference.is_mutable() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected mutable reference.".into(),
                        None,
                        reference.get_span(),
                    ));
                }

                typechecker.analyze_stmt(value)?;

                return Ok(());
            }

            if let (None, Some(source)) = source {
                let source_type: &ThrushType = source.get_value_type()?;

                if !source_type.is_ptr_type() && !source_type.is_mut_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'mut T' type.".into(),
                        None,
                        *span,
                    ));
                }

                if !source.is_mutable() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected mutable reference.".into(),
                        None,
                        source.get_span(),
                    ));
                }

                typechecker.analyze_stmt(value)?;

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

        ThrushStatement::FixedArray { items, kind, span } => {
            if kind.is_void_type() {
                return Err(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "An element is expected.".into(),
                    None,
                    *span,
                ));
            }

            let array_type: &ThrushType = kind.get_fixed_array_base_type();

            items.iter().try_for_each(|item| {
                let item_type: &ThrushType = item.get_value_type()?.get_fixed_array_base_type();

                if let Err(error) = typechecker.validate_types(
                    array_type,
                    item_type,
                    Some(item),
                    None,
                    None,
                    &item.get_span(),
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_stmt(item)
            })?;

            Ok(())
        }

        ThrushStatement::Array {
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

            let array_type: &ThrushType = kind.get_array_base_type();

            items.iter().try_for_each(|item| {
                let item_type: &ThrushType = item.get_value_type()?.get_array_base_type();

                if let Err(error) = typechecker.validate_types(
                    array_type,
                    item_type,
                    Some(item),
                    None,
                    None,
                    &item.get_span(),
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_stmt(item)
            })?;

            Ok(())
        }

        ThrushStatement::Index {
            index_to,
            indexes,
            span,
            ..
        } => {
            if let Some(any_reference) = &index_to.0 {
                let reference: &ThrushStatement = &any_reference.1;

                if !reference.is_allocated_reference() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected a allocated value.".into(),
                        None,
                        *span,
                    ));
                }

                let reference_type: &ThrushType = reference.get_value_type()?;

                if reference_type.is_ptr_type() && !reference_type.is_typed_ptr() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        *span,
                    ));
                } else if reference_type.is_ptr_type()
                    && reference_type.is_typed_ptr()
                    && reference_type.is_all_ptr()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        *span,
                    ));
                }
            }

            if let Some(expr) = &index_to.1 {
                let expr_type: &ThrushType = expr.get_stmt_type()?;

                if expr_type.is_ptr_type() && !expr_type.is_typed_ptr() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        *span,
                    ));
                } else if expr_type.is_ptr_type()
                    && expr_type.is_typed_ptr()
                    && expr_type.is_all_ptr()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        *span,
                    ));
                }
            }

            indexes.iter().try_for_each(|indexe| {
                if !indexe.is_unsigned_integer()? || !indexe.is_moreu32bit_integer()? {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected any unsigned integer value more than or equal to 32 bits.".into(),
                        None,
                        *span,
                    ));
                }

                typechecker.analyze_stmt(indexe)
            })?;

            Ok(())
        }

        ThrushStatement::Property { reference, .. } => {
            let reference_type: &ThrushType = reference.get_value_type()?;
            let reference_span: Span = reference.get_span();

            if !reference_type.is_struct_type()
                && !reference_type.is_mut_struct_type()
                && !reference_type.is_ptr_struct_type()
            {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected reference with a struct type.".into(),
                    None,
                    reference_span,
                ));
            }

            typechecker.analyze_stmt(reference)?;

            Ok(())
        }

        ThrushStatement::Constructor { arguments, .. } => {
            let args: &[(&str, ThrushStatement, ThrushType, u32)] = &arguments.1;

            args.iter().try_for_each(|arg| {
                let expression: &ThrushStatement = &arg.1;
                let expression_span: Span = expression.get_span();

                let target_type: &ThrushType = &arg.2;
                let from_type: &ThrushType = expression.get_value_type()?;

                if let Err(error) = typechecker.validate_types(
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

        ThrushStatement::Call {
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

        ThrushStatement::AsmValue { .. }
        | ThrushStatement::Alloc { .. }
        | ThrushStatement::EnumValue { .. }
        | ThrushStatement::Reference { .. }
        | ThrushStatement::Integer { .. }
        | ThrushStatement::Boolean { .. }
        | ThrushStatement::Str { .. }
        | ThrushStatement::Float { .. }
        | ThrushStatement::Null { .. }
        | ThrushStatement::NullPtr { .. }
        | ThrushStatement::Char { .. }
        | ThrushStatement::Pass { .. } => Ok(()),

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
