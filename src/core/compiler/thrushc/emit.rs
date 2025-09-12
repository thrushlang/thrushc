use std::{fs::write, path::Path, time::Instant};

use inkwell::{module::Module, targets::TargetMachine};

use crate::{
    core::{
        compiler::{
            emitters,
            options::{CompilerFile, CompilerOptions, EmitableUnit, Emited},
            thrushc::{TheThrushCompiler, interrupt},
        },
        console::logging::{self, LoggingType},
    },
    frontends::classical::lexer,
};

pub fn llvm_after_optimization(
    compiler: &mut TheThrushCompiler,
    archive_time: Instant,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file: &CompilerFile,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler_options);

    if compiler_options.contains_emitable(EmitableUnit::LLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            &file.name,
            false,
        ) {
            logging::log(LoggingType::Error, "Failed to emit LLVM bitcode.");
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::LLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, &file.name, false)
        {
            logging::log(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::Assembly) {
        if let Err(error) = emitters::assembler::emit_llvm_assembler(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            &file.name,
            false,
        ) {
            logging::log(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        };

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::Object) {
        if let Err(error) = emitters::obj::emit_llvm_object(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            &file.name,
            false,
        ) {
            logging::log(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn llvm_before_optimization(
    compiler: &mut TheThrushCompiler,
    archive_time: Instant,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file: &CompilerFile,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler_options);

    if compiler_options.contains_emitable(EmitableUnit::RawLLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, &file.name, true)
        {
            logging::log(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::RawLLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            &file.name,
            true,
        ) {
            logging::log(LoggingType::Error, "Failed to emit LLVM bitcode.");
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::RawAssembly) {
        if let Err(error) = emitters::assembler::emit_llvm_assembler(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            &file.name,
            true,
        ) {
            logging::log(LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, archive_time, file)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn after_frontend(
    compiler: &mut TheThrushCompiler,
    build_dir: &Path,
    file: &CompilerFile,
    emited: Emited,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    emitters::cleaner::auto_clean(compiler.options);

    if compiler_options.contains_emitable(EmitableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if lexer::printer::print_to_file(tokens, build_dir, &file.name).is_err() {
                return false;
            }

            return true;
        }
    }

    if compiler_options.contains_emitable(EmitableUnit::AST) {
        if let Emited::Ast(stmts) = emited {
            let _ = write(
                build_dir.join(format!("{}.ast", file.name)),
                format!("{:#?}", stmts),
            );

            return true;
        }
    }

    false
}
