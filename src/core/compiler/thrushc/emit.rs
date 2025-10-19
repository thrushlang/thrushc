use std::{fs::write, path::Path, time::Instant};

use inkwell::{module::Module, targets::TargetMachine};

use crate::{
    core::{
        compiler::{
            emitters,
            options::{CompilationUnit, CompilerOptions, EmitableUnit, Emited},
            thrushc::{ThrushCompiler, interrupt},
        },
        console::logging::{self, LoggingType},
    },
    frontends::classical::lexer,
};

pub fn llvm_after_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file: &CompilationUnit,
    file_time: Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler_options);

    if compiler_options.contains_emitable(EmitableUnit::LLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            file.get_name(),
            false,
        ) {
            logging::print_error(
                LoggingType::Error,
                &format!(
                    "Failed to emit LLVM bitcode for file '{}'.",
                    file.get_path().display()
                ),
            );

            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::LLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, file.get_name(), false)
        {
            logging::print_error(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::Assembly) {
        if let Err(error) = emitters::assembler::emit_llvm_assembler(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            file.get_name(),
            false,
        ) {
            logging::print_error(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        };

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::Object) {
        if let Err(error) = emitters::obj::emit_llvm_object(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            file.get_name(),
            false,
        ) {
            logging::print_error(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn llvm_before_optimization(
    compiler: &mut ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file: &CompilationUnit,
    file_time: Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler_options);

    if compiler_options.contains_emitable(EmitableUnit::RawLLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, file.get_name(), true)
        {
            logging::print_error(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::RawLLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            file.get_name(),
            true,
        ) {
            logging::print_error(
                LoggingType::Error,
                &format!(
                    "Failed to emit LLVM bitcode for file '{}'.",
                    file.get_path().display()
                ),
            );
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::RawAssembly) {
        if let Err(error) = emitters::assembler::emit_llvm_assembler(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            file.get_name(),
            true,
        ) {
            logging::print_error(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file_time, file)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn after_frontend(
    compiler: &mut ThrushCompiler,
    build_dir: &Path,
    file: &CompilationUnit,
    emited: Emited,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler.options);

    if compiler_options.contains_emitable(EmitableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if lexer::printer::print_to_file(tokens, build_dir, file.get_name()).is_err() {
                return false;
            }

            return true;
        }
    }

    if compiler_options.contains_emitable(EmitableUnit::AST) {
        if let Emited::Ast(stmts) = emited {
            let _ = write(
                build_dir.join(format!("{}.ast", file.get_name())),
                format!("{:#?}", stmts),
            );

            return true;
        }
    }

    false
}
