use crate::{backend::llvm::compiler::builtins::Builtin, frontend::semantic::linter::Linter};

pub fn analyze_builtin<'linter>(linter: &mut Linter<'linter>, builtin: &'linter Builtin) {
    match builtin {
        Builtin::MemCpy {
            source,
            destination,
            size,
        } => {
            linter.analyze_ast_expr(source);
            linter.analyze_ast_expr(destination);
            linter.analyze_ast_expr(size);
        }

        Builtin::MemMove {
            source,
            destination,
            size,
        } => {
            linter.analyze_ast_expr(source);
            linter.analyze_ast_expr(destination);
            linter.analyze_ast_expr(size);
        }

        Builtin::MemSet {
            destination,
            new_size,
            size,
        } => {
            linter.analyze_ast_expr(destination);
            linter.analyze_ast_expr(new_size);
            linter.analyze_ast_expr(size);
        }

        _ => (),
    }
}
