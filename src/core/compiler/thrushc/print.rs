use inkwell::module::Module;

use crate::{
    core::compiler::{
        options::{CompilationUnit, CompilerOptions, Emited, PrintableUnit},
        printers,
        thrushc::ThrushCompiler,
    },
    frontends::classical::lexer,
};

#[inline]
pub fn llvm_before_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    file: &CompilationUnit,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::RawLLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, file.get_name(), true);
        return true;
    }

    false
}

#[inline]
pub fn llvm_after_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    file: &CompilationUnit,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::LLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, file.get_name(), false);

        return true;
    }

    false
}

pub fn after_frontend(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    emited: Emited,
) -> bool {
    let options: &CompilerOptions = compiler.get_options();

    if options.contains_printable(PrintableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if lexer::printer::print_to_stdout_fine(tokens, file.get_name()).is_err() {
                return false;
            }

            return true;
        }
    }

    false
}
