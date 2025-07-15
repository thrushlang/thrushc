use std::path::{Path, PathBuf};

use inkwell::{
    module::Module,
    support::LLVMString,
    targets::{FileType, TargetMachine},
};

use crate::core::{
    compiler::{options::CompilerOptions, thrushc::TheThrushCompiler},
    utils::rand,
};

pub fn emit_llvm_object(
    compiler: &TheThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.ofuscate_archive_names();

    let objects_base_path: PathBuf = build_dir.join("emit").join("obj");

    if !objects_base_path.exists() {
        let _ = std::fs::create_dir_all(&objects_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let object_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.o",
            optimization_name_modifier,
            rand::generate_random_string(),
            file_name
        )
    } else {
        format!("{}{}.o", optimization_name_modifier, file_name)
    };

    let object_file_path: PathBuf = objects_base_path.join(object_file_name);

    target_machine.write_to_file(llvm_module, FileType::Object, &object_file_path)?;

    Ok(())
}
