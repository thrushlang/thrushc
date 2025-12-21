use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstGetType};
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    builtin: &'type_checker crate::middle_end::mir::builtins::ThrushBuiltin,
) -> Result<(), CompilationIssue> {
    match builtin {
        crate::middle_end::mir::builtins::ThrushBuiltin::MemSet {
            dst,
            new_size,
            size,
            ..
        } => self::validate_memset(typechecker, dst, new_size, size),

        crate::middle_end::mir::builtins::ThrushBuiltin::MemMove { dst, src, size, .. } => {
            self::validate_memmove(typechecker, dst, src, size)
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::MemCpy { dst, src, size, .. } => {
            self::validate_memcpy(typechecker, dst, src, size)
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::Halloc { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AlignOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::SizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AbiSizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::BitSizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AbiAlignOf { .. } => Ok(()),
    }
}

pub fn validate_memmove<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,

    destination: &'type_checker Ast,
    source: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    let source_type: &Type = source.get_value_type()?;
    let source_span: Span = source.get_span();

    let destination_type: &Type = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let size_type: &Type = size.get_value_type()?;
    let size_span: Span = size.get_span();

    if !source_type.is_ptr_type() && !source_type.is_address_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', memory address 'addr' type, got '{}' type.", source_type),
            None,
            source_span,
        ));
    }

    if !destination_type.is_ptr_type() && !destination_type.is_address_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', memory address 'addr' type, got '{}' type.", destination_type)
                ,
            None,
            destination_span,
        ));
    }

    if !size_type.is_unsigned_integer_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected unsigned integer type, got '{}' type.", size_type),
            None,
            size_span,
        ));
    }

    typechecker.analyze_expr(source)?;
    typechecker.analyze_expr(destination)?;
    typechecker.analyze_expr(size)?;

    Ok(())
}

pub fn validate_memcpy<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,

    destination: &'type_checker Ast,
    source: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    let source_type: &Type = source.get_value_type()?;
    let source_span: Span = source.get_span();

    let destination_type: &Type = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let size_type: &Type = size.get_value_type()?;
    let size_span: Span = size.get_span();

    if !source_type.is_ptr_type() && !source_type.is_address_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', memory address 'addr'  type, got '{}' type.", source_type),
            None,
            source_span,
        ));
    }

    if !destination_type.is_ptr_type() && !destination_type.is_address_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            "Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', memory address 'addr' type."
                .into(),
            None,
            destination_span,
        ));
    }

    if size_type.is_unsigned_integer_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected unsigned integer type, got '{}' type.", size_type),
            None,
            size_span,
        ));
    }

    typechecker.analyze_expr(source)?;
    typechecker.analyze_expr(destination)?;
    typechecker.analyze_expr(size)?;

    Ok(())
}

pub fn validate_memset<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,

    destination: &'type_checker Ast,
    new_size: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    let destination_type: &Type = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let new_size_type: &Type = new_size.get_value_type()?;
    let new_size_span: Span = new_size.get_span();

    let size_type: &Type = size.get_value_type()?;
    let size_span: Span = size.get_span();

    if !destination_type.is_ptr_type() && !destination_type.is_address_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected raw typed pointer 'ptr[T]', raw pointer 'ptr', memory address 'addr' type, got '{}' type.", size_type),
            None,
            destination_span,
        ));
    }

    if !new_size_type.is_unsigned_integer_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!(
                "Expected unsigned integer type, got '{}' type.",
                new_size_type
            ),
            None,
            new_size_span,
        ));
    }

    if !size_type.is_unsigned_integer_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            format!("Expected unsigned integer type, got '{}' type.", size_type),
            None,
            size_span,
        ));
    }

    typechecker.analyze_expr(destination)?;
    typechecker.analyze_expr(new_size)?;
    typechecker.analyze_expr(size)?;

    Ok(())
}
