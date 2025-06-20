use crate::{backend::llvm::compiler::builtins::Builtin, frontend::semantic::linter::Linter};

pub fn analyze_builtin<'linter>(linter: &mut Linter<'linter>, builtin: &'linter Builtin) {
    match builtin {
        Builtin::MemCpy {
            source,
            destination,
            size,
        } => {
            linter.analyze_stmt(source);
            linter.analyze_stmt(destination);
            linter.analyze_stmt(size);
        }

        Builtin::MemMove {
            source,
            destination,
            size,
        } => {
            linter.analyze_stmt(source);
            linter.analyze_stmt(destination);
            linter.analyze_stmt(size);
        }

        Builtin::MemSet {
            destination,
            new_size,
            size,
        } => {
            linter.analyze_stmt(destination);
            linter.analyze_stmt(new_size);
            linter.analyze_stmt(size);
        }

        _ => (),
    }
}
