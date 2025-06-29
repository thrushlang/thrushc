use crate::{
    backend::llvm::compiler::builtins::Builtin,
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{ast::Ast, lexer::ThrushType},
    },
};

pub fn validate_builtin<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    builtin: &'type_checker Builtin,
    span: Span,
) -> Result<(), ThrushCompilerIssue> {
    match builtin {
        Builtin::MemSet {
            destination,
            new_size,
            size,
        } => validate_memset(typechecker, destination, new_size, size),

        Builtin::MemMove {
            destination,
            source,
            size,
        } => validate_memmove(typechecker, destination, source, size),

        Builtin::MemCpy {
            destination,
            source,
            size,
        } => validate_memcpy(typechecker, destination, source, size),

        _ => {
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

pub fn validate_memmove<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    destination: &'type_checker Ast,
    source: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    let source_type: &ThrushType = source.get_value_type()?;
    let source_span: Span = source.get_span();

    let destination_type: &ThrushType = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let size_span: Span = size.get_span();

    if !source_type.is_ptr_type() && !source_type.is_address_type() && !source_type.is_mut_type() {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected 'ptr[T]', 'ptr', 'addr', or 'mut T' type.".into(),
            None,
            source_span,
        ));
    }

    if !destination_type.is_ptr_type()
        && !destination_type.is_address_type()
        && !destination_type.is_mut_type()
    {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected 'ptr[T]', 'ptr', 'addr', or 'mut T' type.".into(),
            None,
            destination_span,
        ));
    }

    if !size.is_unsigned_integer()? {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected any unsigned integer value.".into(),
            None,
            size_span,
        ));
    }

    typechecker.analyze_ast(source)?;
    typechecker.analyze_ast(destination)?;
    typechecker.analyze_ast(size)?;

    Ok(())
}

pub fn validate_memcpy<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    destination: &'type_checker Ast,
    source: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    let source_type: &ThrushType = source.get_value_type()?;
    let source_span: Span = source.get_span();

    let destination_type: &ThrushType = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let size_span: Span = size.get_span();

    if !source_type.is_ptr_type() && !source_type.is_address_type() && !source_type.is_mut_type() {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected 'ptr[T]', 'ptr', 'addr', or 'mut T' type.".into(),
            None,
            source_span,
        ));
    }

    if !destination_type.is_ptr_type()
        && !destination_type.is_address_type()
        && !destination_type.is_mut_type()
    {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected 'ptr[T]', 'ptr', 'addr', or 'mut T' type.".into(),
            None,
            destination_span,
        ));
    }

    if !size.is_unsigned_integer()? {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected any unsigned integer value.".into(),
            None,
            size_span,
        ));
    }

    typechecker.analyze_ast(source)?;
    typechecker.analyze_ast(destination)?;
    typechecker.analyze_ast(size)?;

    Ok(())
}

pub fn validate_memset<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    destination: &'type_checker Ast,
    new_size: &'type_checker Ast,
    size: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    let destination_type: &ThrushType = destination.get_value_type()?;
    let destination_span: Span = destination.get_span();

    let new_size_span: Span = new_size.get_span();
    let size_span: Span = size.get_span();

    if !destination_type.is_ptr_type()
        && !destination_type.is_address_type()
        && !destination_type.is_mut_type()
    {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected 'ptr[T]', 'ptr', 'addr', or 'mut T' type.".into(),
            None,
            destination_span,
        ));
    }

    if !new_size.is_unsigned_integer()? {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected any unsigned integer value.".into(),
            None,
            new_size_span,
        ));
    }

    if !size.is_unsigned_integer()? {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            "Expected any unsigned integer value.".into(),
            None,
            size_span,
        ));
    }

    typechecker.analyze_ast(destination)?;
    typechecker.analyze_ast(new_size)?;
    typechecker.analyze_ast(size)?;

    Ok(())
}
