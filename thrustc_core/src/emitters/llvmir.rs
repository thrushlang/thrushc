use inkwell::{module::Module, support::LLVMString};
use thrustc_options::CompilerOptions;

use crate::{ThrustCompiler, utils};

pub fn emit_llvm_ir(
    compiler: &ThrustCompiler,
    llvm_module: &Module,
    build_dir: &std::path::Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let need_obfuscation: bool = compiler_options.need_obfuscate_archive_names();

    let llvmir_base_path: std::path::PathBuf = build_dir.join("emit").join("llvm-ir");

    if !llvmir_base_path.exists() {
        let _ = std::fs::create_dir_all(&llvmir_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let llvmir_file_name: String = if need_obfuscation {
        format!(
            "{}{}_{}.ll",
            optimization_name_modifier,
            utils::generate_random_string(thrustc_constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.ll", optimization_name_modifier, file_name)
    };

    let llvmir_file_path: std::path::PathBuf = llvmir_base_path.join(llvmir_file_name);

    llvm_module.print_to_file(&llvmir_file_path)?;

    Ok(())
}
