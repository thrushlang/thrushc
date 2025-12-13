use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::Analyzer;

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    builtin: &'analyzer crate::middle_end::mir::builtins::ThrushBuiltin,
) -> Result<(), CompilationIssue> {
    match builtin {
        crate::middle_end::mir::builtins::ThrushBuiltin::MemSet {
            dst,
            new_size,
            size,
            ..
        } => {
            analyzer.analyze_expr(dst)?;
            analyzer.analyze_expr(new_size)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::MemMove { dst, src, size, .. } => {
            analyzer.analyze_expr(dst)?;
            analyzer.analyze_expr(src)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::MemCpy { dst, src, size, .. } => {
            analyzer.analyze_expr(dst)?;
            analyzer.analyze_expr(src)?;
            analyzer.analyze_expr(size)?;

            Ok(())
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::Halloc { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AlignOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::SizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AbiSizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::BitSizeOf { .. }
        | crate::middle_end::mir::builtins::ThrushBuiltin::AbiAlignOf { .. } => Ok(()),
    }
}
