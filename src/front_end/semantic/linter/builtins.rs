use crate::back_end::llvm_codegen::builtins::Builtin;
use crate::front_end::semantic::linter::Linter;

pub fn analyze<'linter>(linter: &mut Linter<'linter>, builtin: &'linter Builtin) {
    match builtin {
        Builtin::MemCpy {
            source,
            destination,
            size,
            ..
        } => {
            linter.analyze_expr(source);
            linter.analyze_expr(destination);
            linter.analyze_expr(size);
        }

        Builtin::MemMove {
            source,
            destination,
            size,
            ..
        } => {
            linter.analyze_expr(source);
            linter.analyze_expr(destination);
            linter.analyze_expr(size);
        }

        Builtin::MemSet {
            destination,
            new_size,
            size,
            ..
        } => {
            linter.analyze_expr(destination);
            linter.analyze_expr(new_size);
            linter.analyze_expr(size);
        }

        _ => (),
    }
}
