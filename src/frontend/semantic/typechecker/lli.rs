use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{
            lexer::{ThrushType, traits::ThrushTypePointerExtensions},
            parser::stmts::stmt::ThrushStatement,
        },
    },
};

pub fn validate_lli<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::LLI {
            name,
            kind: lli_type,
            value: lli_value,
            span,
            ..
        } => {
            typechecker.symbols.new_lli(name, (lli_type, *span));

            let lli_value_type: &ThrushType = lli_value.get_value_type()?;

            if lli_type.is_void_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            if let Err(mismatch_type_error) =
                typechecker.validate_types(lli_type, lli_value_type, Some(lli_value), None, None, span)
            {
                typechecker.add_error(mismatch_type_error);
            }

            if let Err(type_error) = typechecker.analyze_stmt(lli_value) {
                typechecker.add_error(type_error);
            }

            Ok(())
        }

        ThrushStatement::Load { value, .. } => {
            if let Some(any_reference) = &value.0 {
                let reference: &ThrushStatement = &any_reference.1;

                let reference_type: &ThrushType = reference.get_value_type()?;
                let reference_span: Span = reference.get_span();

                if !reference_type.is_ptr_type() && !reference_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        reference_span,
                    ));
                }

                typechecker.analyze_stmt(reference)?;
            }

            if let Some(expr) = &value.1 {
                let expr_type: &ThrushType = expr.get_value_type()?;
                let expr_span: Span = expr.get_span();

                if !expr_type.is_ptr_type() && !expr_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr' or 'addr' type.".into(),
                        None,
                        expr_span,
                    ));
                }

                typechecker.analyze_stmt(expr)?;
            }

            Ok(())
        }

        ThrushStatement::Address {
            address_to,
            indexes,
            span,
            ..
        } => {
            if let Some(reference_any) = &address_to.0 {
                let reference: &ThrushStatement = &reference_any.1;

                let reference_type: &ThrushType = reference.get_value_type()?;
                let reference_span: Span = reference.get_span();

                if !reference_type.is_ptr_type() && !reference_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        reference_span,
                    ));
                }

                if reference_type.is_ptr_type() && !reference_type.is_typed_ptr() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        reference_span,
                    ));
                } else if reference_type.is_ptr_type()
                    && reference_type.is_typed_ptr()
                    && !reference_type.is_ptr_struct_type()
                    && !reference_type.is_ptr_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        reference_span,
                    ));
                }

                if reference_type.is_address_type() {
                    typechecker.add_warning(ThrushCompilerIssue::Warning(
                        "Undefined behavior".into(), 
                        "*Maybe* this value at runtime causes undefined behavior because it is anything at runtime, and memory calculation needs valid pointers or deep types.".into(), 
                       reference_span
                    ));
                }
            }

            if let Some(expr) = &address_to.1 {
                let expr_type: &ThrushType = expr.get_value_type()?;
                let expr_span: Span = expr.get_span();

                if !expr_type.is_ptr_type() && !expr_type.is_address_type() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', or 'addr' type.".into(),
                        None,
                        expr_span,
                    ));
                }

                if expr_type.is_ptr_type() && !expr_type.is_typed_ptr() {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer ptr<T>.".into(),
                        None,
                        expr_span,
                    ));
                } else if expr_type.is_ptr_type()
                    && expr_type.is_typed_ptr()
                    && !expr_type.is_ptr_struct_type()
                    && !expr_type.is_ptr_fixed_array_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected raw typed pointer type with deep type.".into(),
                        None,
                        expr_span,
                    ));
                }

                if expr_type.is_address_type() {
                    typechecker.add_warning(ThrushCompilerIssue::Warning(
                        "Undefined behavior".into(), 
                        "*Maybe* this value at runtime causes undefined behavior because it is anything at runtime, and memory calculation needs valid pointers or deep types.".into(), 
                        expr_span
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

                Ok(())
            })?;

            Ok(())
        }

        ThrushStatement::Write {
            write_to,
            write_value,
            write_type,
            ..
        } => {
            if let Some(any_reference) = &write_to.0 {
                let reference: &ThrushStatement = &any_reference.1;
                let reference_type: &ThrushType = reference.get_value_type()?;
                let reference_span: Span = reference.get_span();

                if !reference_type.is_ptr_type()
                    && !reference_type.is_address_type()
                    && !reference_type.is_mut_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', 'addr', or 'mut T' type.".into(),
                        None,
                        reference_span,
                    ));
                }
            }

            if let Some(expr) = &write_to.1 {
                let expr_type: &ThrushType = expr.get_value_type()?;
                let expr_span: Span = expr.get_span();

                if !expr_type.is_ptr_type()
                    && !expr_type.is_address_type()
                    && !expr_type.is_mut_type()
                {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        "Expected 'ptr<T>', 'ptr', 'addr', or 'mut T' type.".into(),
                        None,
                        expr_span,
                    ));
                }
            }

            let write_value_type: &ThrushType = write_value.get_value_type()?;
            let write_value_span: Span = write_value.get_span();

            if let Err(error) = typechecker.validate_types(
                write_type,
                write_value_type,
                Some(write_value),
                None,
                None,
                &write_value_span,
            ) {
                typechecker.add_error(error);
            }

            Ok(())
        }

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
