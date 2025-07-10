use std::path::{Path, PathBuf};

use inkwell::{
    module::Module,
    support::LLVMString,
    targets::{FileType, TargetMachine},
};

use crate::core::utils::rand;

pub fn emit_llvm_assembler(
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let assembler_base_path: PathBuf = build_dir.join("emit").join("assembler");

    if !assembler_base_path.exists() {
        let _ = std::fs::create_dir_all(&assembler_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let assembler_file_name: String = format!(
        "{}{}_{}.s",
        optimization_name_modifier,
        rand::generate_random_string(),
        file_name
    );
    let assembler_file_path: PathBuf = assembler_base_path.join(assembler_file_name);

    target_machine.write_to_file(llvm_module, FileType::Assembly, &assembler_file_path)?;

    Ok(())
}
