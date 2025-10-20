use crate::{
    backend::llvm::compiler::builtins::Builtin,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{semantic::analyzer::Analyzer, types::ast::Ast},
};

pub fn validate_builtin<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    builtin: &'analyzer Builtin,
) -> Result<(), ThrushCompilerIssue> {
    match builtin {
        Builtin::MemSet {
            destination,
            new_size,
            size,
            ..
        } => self::validate_memset(analyzer, destination, new_size, size),

        Builtin::MemMove {
            destination,
            source,
            size,
            ..
        } => self::validate_memmove(analyzer, destination, source, size),

        Builtin::MemCpy {
            destination,
            source,
            size,
            ..
        } => self::validate_memcpy(analyzer, destination, source, size),

        Builtin::Halloc { .. } | Builtin::AlignOf { .. } | Builtin::SizeOf { .. } => Ok(()),
    }
}

pub fn validate_memmove<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,

    destination: &'analyzer Ast,
    source: &'analyzer Ast,
    size: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    analyzer.analyze_expr(source)?;
    analyzer.analyze_expr(destination)?;
    analyzer.analyze_expr(size)?;

    Ok(())
}

pub fn validate_memcpy<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,

    destination: &'analyzer Ast,
    source: &'analyzer Ast,
    size: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    analyzer.analyze_expr(source)?;
    analyzer.analyze_expr(destination)?;
    analyzer.analyze_expr(size)?;

    Ok(())
}

pub fn validate_memset<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,

    destination: &'analyzer Ast,
    new_size: &'analyzer Ast,
    size: &'analyzer Ast,
) -> Result<(), ThrushCompilerIssue> {
    analyzer.analyze_expr(destination)?;
    analyzer.analyze_expr(new_size)?;
    analyzer.analyze_expr(size)?;

    Ok(())
}
