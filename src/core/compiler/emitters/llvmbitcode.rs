use std::path::{Path, PathBuf};

use inkwell::module::Module;

use crate::core::utils::rand;

pub fn emit_llvm_bitcode(
    llvm_module: &Module,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> bool {
    let bitcode_base_path: PathBuf = build_dir.join("emit").join("llvm-bitcode");

    if !bitcode_base_path.exists() {
        let _ = std::fs::create_dir_all(&bitcode_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let bitcode_file_name: String = format!(
        "{}{}_{}.bc",
        optimization_name_modifier,
        rand::generate_random_string(),
        file_name
    );

    let bitcode_file_path: PathBuf = bitcode_base_path.join(bitcode_file_name);

    llvm_module.write_bitcode_to_path(&bitcode_file_path)
}
