use inkwell::{
    module::Module,
    support::LLVMString,
    targets::{FileType, TargetMachine},
};
use thrushc_options::CompilerOptions;

use crate::{ThrushCompiler, utils};

pub fn emit_llvm_object(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &std::path::Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let need_obfuscation: bool = compiler_options.need_obfuscate_archive_names();

    let objects_base_path: std::path::PathBuf = build_dir.join("emit").join("obj");

    if !objects_base_path.exists() {
        let _ = std::fs::create_dir_all(&objects_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let object_file_name: String = if need_obfuscation {
        format!(
            "{}{}_{}.o",
            optimization_name_modifier,
            utils::generate_random_string(thrushc_constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.o", optimization_name_modifier, file_name)
    };

    let object_file_path: std::path::PathBuf = objects_base_path.join(object_file_name);

    target_machine.write_to_file(llvm_module, FileType::Object, &object_file_path)?;

    Ok(())
}
