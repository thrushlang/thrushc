use std::path::{Path, PathBuf};

use inkwell::{
    module::Module,
    targets::{FileType, TargetMachine},
};

use crate::core::utils::rand;

pub fn emit_llvm_object(
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> bool {
    let objects_base_path: PathBuf = build_dir.join("emit").join("obj");

    if !objects_base_path.exists() {
        let _ = std::fs::create_dir_all(&objects_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let object_file_name: String = format!(
        "{}{}_{}.o",
        optimization_name_modifier,
        rand::generate_random_string(),
        file_name
    );

    let object_file_path: PathBuf = objects_base_path.join(object_file_name);

    target_machine
        .write_to_file(llvm_module, FileType::Object, &object_file_path)
        .is_err()
}
