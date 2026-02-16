use inkwell::module::Module;
use inkwell::targets::TargetMachine;
use thrustc_options::{CompilationUnit, CompilerOptions, EmitableUnit, Emited};

use crate::{ThrustCompiler, emitters, interrupt};

pub fn llvm_after_optimization(
    compiler: &mut ThrustCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &std::path::Path,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_emitable(EmitableUnit::LLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            file.get_name(),
            false,
        ) {
            thrustc_logging::print_error(
                thrustc_logging::LoggingType::Error,
                &format!(
                    "Failed to emit LLVM bitcode for file '{}'.",
                    file.get_path().display()
                ),
            );

            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::LLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, file.get_name(), false)
        {
            thrustc_logging::print_error(thrustc_logging::LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
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
            thrustc_logging::print_error(thrustc_logging::LoggingType::Error, error);
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        };

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::Object) {
        if let Err(error) = emitters::objfile::emit_llvm_object(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            file.get_name(),
            false,
        ) {
            thrustc_logging::print_error(thrustc_logging::LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn llvm_before_optimization(
    compiler: &mut ThrustCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &std::path::Path,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<bool, ()> {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_emitable(EmitableUnit::UnOptLLVMIR) {
        if let Err(error) =
            emitters::llvmir::emit_llvm_ir(compiler, llvm_module, build_dir, file.get_name(), true)
        {
            thrustc_logging::print_error(thrustc_logging::LoggingType::Error, &error.to_string());
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::UnOptLLVMBitcode) {
        if !emitters::llvmbitcode::emit_llvm_bitcode(
            compiler,
            llvm_module,
            build_dir,
            file.get_name(),
            true,
        ) {
            thrustc_logging::print_error(
                thrustc_logging::LoggingType::Error,
                &format!(
                    "Failed to emit LLVM bitcode for file '{}'.",
                    file.get_path().display()
                ),
            );
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    if compiler_options.contains_emitable(EmitableUnit::UnOptAssembly) {
        if let Err(error) = emitters::assembler::emit_llvm_assembler(
            compiler,
            llvm_module,
            target_machine,
            build_dir,
            file.get_name(),
            true,
        ) {
            thrustc_logging::print_error(thrustc_logging::LoggingType::Error, error);
            interrupt::archive_compilation_unit(compiler, file, file_time)?;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn after_frontend(
    compiler: &mut ThrustCompiler,
    build_dir: &std::path::Path,
    file: &CompilationUnit,
    emited: Emited,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();

    if compiler_options.contains_emitable(EmitableUnit::Tokens) {
        if let Emited::Tokens(tokens) = emited {
            if thrustc_lexer::printer::print_to_file(tokens, build_dir, file.get_name()).is_err() {
                return false;
            }

            return true;
        }
    }

    if compiler_options.contains_emitable(EmitableUnit::AST) {
        if let Emited::Ast(stmts) = emited {
            let _ = std::fs::write(
                build_dir.join(format!("{}.ast", file.get_name())),
                format!("{:#?}", stmts),
            );

            return true;
        }
    }

    false
}
