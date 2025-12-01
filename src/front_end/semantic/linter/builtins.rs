use crate::front_end::semantic::linter::Linter;

pub fn analyze<'linter>(
    linter: &mut Linter<'linter>,
    builtin: &'linter crate::middle_end::mir::builtins::ThrushBuiltin,
) {
    match builtin {
        crate::middle_end::mir::builtins::ThrushBuiltin::MemCpy { src, dst, size, .. } => {
            linter.analyze_expr(src);
            linter.analyze_expr(dst);
            linter.analyze_expr(size);
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::MemMove { src, dst, size, .. } => {
            linter.analyze_expr(src);
            linter.analyze_expr(dst);
            linter.analyze_expr(size);
        }

        crate::middle_end::mir::builtins::ThrushBuiltin::MemSet {
            dst,
            new_size,
            size,
            ..
        } => {
            linter.analyze_expr(dst);
            linter.analyze_expr(new_size);
            linter.analyze_expr(size);
        }

        _ => (),
    }
}
