use std::path::Path;
use std::path::PathBuf;

use inkwell::module::Module;
use inkwell::support::LLVMString;

use crate::core::compiler::options::CompilerOptions;
use crate::core::compiler::thrushc::ThrushCompiler;
use crate::core::constants;
use crate::core::utils::rand;

pub fn emit_llvm_ir(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.need_obfuscate_archive_names();

    let llvmir_base_path: PathBuf = build_dir.join("emit").join("llvm-ir");

    if !llvmir_base_path.exists() {
        let _ = std::fs::create_dir_all(&llvmir_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let llvmir_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.ll",
            optimization_name_modifier,
            rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.ll", optimization_name_modifier, file_name)
    };

    let llvmir_file_path: PathBuf = llvmir_base_path.join(llvmir_file_name);

    llvm_module.print_to_file(&llvmir_file_path)?;

    Ok(())
}
