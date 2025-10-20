use std::path::{Path, PathBuf};

use inkwell::{
    module::Module,
    support::LLVMString,
    targets::{FileType, TargetMachine},
};

use crate::core::{
    compiler::{options::CompilerOptions, thrushc::ThrushCompiler},
    constants,
    utils::rand,
};

pub fn emit_llvm_assembler(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.need_obfuscate_archive_names();

    let assembler_base_path: PathBuf = build_dir.join("emit").join("assembler");

    if !assembler_base_path.exists() {
        let _ = std::fs::create_dir_all(&assembler_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let assembler_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.s",
            optimization_name_modifier,
            rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.s", optimization_name_modifier, file_name)
    };

    let assembler_file_path: PathBuf = assembler_base_path.join(assembler_file_name);

    target_machine.write_to_file(llvm_module, FileType::Assembly, &assembler_file_path)?;

    Ok(())
}
