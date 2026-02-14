use inkwell::module::Module;
use inkwell::targets::TargetMachine;

use thrushc_options::{CompilationUnit, CompilerOptions, Emited, PrintableUnit};

use crate::{ThrushCompiler, interrupt, printers};

#[inline]
pub fn llvm_before_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::UnOptLLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, file.get_name(), true);
        return Ok(true);
    }

    if compiler_options.contains_printable(PrintableUnit::UnOptAssembly) {
        if let Err(error) = printers::assembler::print_llvm_assembler(
            compiler,
            target_machine,
            llvm_module,
            file.get_name(),
            true,
        ) {
            thrushc_logging::print_error(thrushc_logging::LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    Ok(false)
}

#[inline]
pub fn llvm_after_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::LLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, file.get_name(), false);
        return Ok(true);
    }

    if compiler_options.contains_printable(PrintableUnit::Assembly) {
        if let Err(error) = printers::assembler::print_llvm_assembler(
            compiler,
            target_machine,
            llvm_module,
            file.get_name(),
            false,
        ) {
            thrushc_logging::print_error(thrushc_logging::LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    Ok(false)
}

#[inline]
pub fn after_frontend(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    emited: Emited,
) -> bool {
    let options: &CompilerOptions = compiler.get_options();

    if options.contains_printable(PrintableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if thrushc_lexer::printer::print_to_stdout_fine(tokens, file.get_name()).is_err() {
                return false;
            }

            return true;
        }
    }

    false
}
