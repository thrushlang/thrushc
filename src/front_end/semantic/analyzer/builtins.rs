use crate::back_end::llvm::compiler::builtins::Builtin;

use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::Analyzer;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    builtin: &'analyzer Builtin,
) -> Result<(), CompilationIssue> {
    match builtin {
        Builtin::MemSet {
            destination,
            new_size,
            size,
            ..
        } => {
            analyzer.analyze_expr(destination)?;
            analyzer.analyze_expr(new_size)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        Builtin::MemMove {
            destination,
            source,
            size,
            ..
        } => {
            analyzer.analyze_expr(source)?;
            analyzer.analyze_expr(destination)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        Builtin::MemCpy {
            destination,
            source,
            size,
            ..
        } => {
            analyzer.analyze_expr(source)?;
            analyzer.analyze_expr(destination)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        Builtin::Halloc { .. } | Builtin::AlignOf { .. } | Builtin::SizeOf { .. } => Ok(()),
    }
}
