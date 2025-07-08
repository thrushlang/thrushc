use std::path::{Path, PathBuf};

use inkwell::module::Module;

use crate::core::utils::rand;

pub fn emit_llvm_ir(
    llvm_module: &Module,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> bool {
    let llvmir_base_path: PathBuf = build_dir.join("emit").join("llvm-ir");

    if !llvmir_base_path.exists() {
        let _ = std::fs::create_dir_all(&llvmir_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let llvmir_file_name: String = format!(
        "{}{}_{}.ll",
        optimization_name_modifier,
        rand::generate_random_string(),
        file_name
    );

    let llvmir_file_path: PathBuf = llvmir_base_path.join(llvmir_file_name);

    llvm_module.print_to_file(&llvmir_file_path).is_err()
}
