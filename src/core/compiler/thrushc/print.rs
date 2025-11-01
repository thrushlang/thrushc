use std::time::Instant;

use inkwell::{module::Module, targets::TargetMachine};

use crate::{
    core::{
        compiler::{
            options::{CompilationUnit, CompilerOptions, Emited, PrintableUnit},
            printers,
            thrushc::{ThrushCompiler, interrupt},
        },
        console::logging::{self, LoggingType},
    },
    frontend::lexer,
};

#[inline]
pub fn llvm_before_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    file: &CompilationUnit,
    file_time: Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_printable(PrintableUnit::RawLLVMIR) {
        printers::llvmir::print_llvm_ir(compiler, llvm_module, file.get_name(), true);
        return Ok(true);
    }

    if compiler_options.contains_printable(PrintableUnit::RawAssembly) {
        if let Err(error) = printers::assembler::print_llvm_assembler(
            compiler,
            target_machine,
            llvm_module,
            file.get_name(),
            true,
        ) {
            logging::print_error(LoggingType::Error, &error.to_string());
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
    file_time: Instant,
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
            logging::print_error(LoggingType::Error, &error.to_string());
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
            if lexer::printer::print_to_stdout_fine(tokens, file.get_name()).is_err() {
                return false;
            }

            return true;
        }
    }

    false
}
