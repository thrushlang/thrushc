use std::path::Path;
use std::path::PathBuf;

use inkwell::module::Module;
use inkwell::support::LLVMString;
use inkwell::targets::FileType;
use inkwell::targets::TargetMachine;

use crate::core::compiler::options::CompilerOptions;
use crate::core::compiler::thrushc::ThrushCompiler;
use crate::core::constants;
use crate::core::utils::rand;

pub fn emit_llvm_object(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let need_obfuscation: bool = compiler_options.need_obfuscate_archive_names();

    let objects_base_path: PathBuf = build_dir.join("emit").join("obj");

    if !objects_base_path.exists() {
        let _ = std::fs::create_dir_all(&objects_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let object_file_name: String = if need_obfuscation {
        format!(
            "{}{}_{}.o",
            optimization_name_modifier,
            rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.o", optimization_name_modifier, file_name)
    };

    let object_file_path: PathBuf = objects_base_path.join(object_file_name);

    target_machine.write_to_file(llvm_module, FileType::Object, &object_file_path)?;

    Ok(())
}
