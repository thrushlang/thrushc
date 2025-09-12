use inkwell::module::Module;

use crate::{
    core::compiler::{
        options::{CompilerFile, CompilerOptions, Emited, PrintableUnit},
        printers,
        thrushc::TheThrushCompiler,
    },
    frontends::classical::lexer,
};

#[inline]
pub fn llvm_before_optimization(
    compiler: &mut TheThrushCompiler,
    llvm_module: &Module,
    file: &CompilerFile,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::RawLLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, &file.name, true);
        return true;
    }

    false
}

#[inline]
pub fn llvm_after_optimization(
    compiler: &mut TheThrushCompiler,
    llvm_module: &Module,
    file: &CompilerFile,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::LLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, &file.name, false);

        return true;
    }

    false
}

pub fn after_frontend(
    compiler: &mut TheThrushCompiler,

    file: &CompilerFile,
    emited: Emited,
) -> bool {
    let options: &CompilerOptions = compiler.get_options();

    if options.contains_printable(PrintableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if lexer::printer::print_to_stdout_fine(tokens, &file.name).is_err() {
                return false;
            }

            return true;
        }
    }

    false
}
