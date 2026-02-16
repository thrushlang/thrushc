use inkwell::{
    module::Module,
    targets::{FileType, TargetMachine},
};
use thrustc_options::CompilerOptions;

use crate::{ThrustCompiler, utils};

pub fn emit_llvm_assembler(
    compiler: &ThrustCompiler,
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &std::path::Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), &'static str> {
    if !target_machine.get_target().has_asm_backend() {
        return Err(
            "The backend doesn't support emitting readable assembly; aborting assembly emission.",
        );
    }

    let compiler_options: &CompilerOptions = compiler.get_options();
    let need_obfuscation: bool = compiler_options.need_obfuscate_archive_names();

    let assembler_base_path: std::path::PathBuf = build_dir.join("emit").join("assembler");

    if !assembler_base_path.exists() {
        let _ = std::fs::create_dir_all(&assembler_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let assembler_file_name: String = if need_obfuscation {
        format!(
            "{}{}_{}.s",
            optimization_name_modifier,
            utils::generate_random_string(thrustc_constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.s", optimization_name_modifier, file_name)
    };

    let assembler_file_path: std::path::PathBuf = assembler_base_path.join(assembler_file_name);

    target_machine
        .write_to_file(llvm_module, FileType::Assembly, &assembler_file_path)
        .map_err(|_| "Failed to compile the readable assembler!")?;

    Ok(())
}
