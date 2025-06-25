use crate::{backend::llvm::compiler::builtins::Builtin, frontend::semantic::linter::Linter};

pub fn analyze_builtin<'linter>(linter: &mut Linter<'linter>, builtin: &'linter Builtin) {
    match builtin {
        Builtin::MemCpy {
            source,
            destination,
            size,
        } => {
            linter.analyze_ast(source);
            linter.analyze_ast(destination);
            linter.analyze_ast(size);
        }

        Builtin::MemMove {
            source,
            destination,
            size,
        } => {
            linter.analyze_ast(source);
            linter.analyze_ast(destination);
            linter.analyze_ast(size);
        }

        Builtin::MemSet {
            destination,
            new_size,
            size,
        } => {
            linter.analyze_ast(destination);
            linter.analyze_ast(new_size);
            linter.analyze_ast(size);
        }

        _ => (),
    }
}
