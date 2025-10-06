use crate::core::{
    compiler::{options::CompilerOptions, thrushc::ThrushCompiler},
    utils::{limits, rand},
};

use inkwell::module::Module;

use std::path::{Path, PathBuf};

pub fn emit_llvm_bitcode(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    build_dir: &Path,
    file_name: &str,
    unoptimized: bool,
) -> bool {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.need_obfuscate_archive_names();

    let bitcode_base_path: PathBuf = build_dir.join("emit").join("llvm-bitcode");

    if !bitcode_base_path.exists() {
        let _ = std::fs::create_dir_all(&bitcode_base_path);
    }

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let bitcode_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.bc",
            optimization_name_modifier,
            rand::generate_random_string(limits::HARD_FILE_NAME_OBFUSCATION),
            file_name
        )
    } else {
        format!("{}{}.bc", optimization_name_modifier, file_name)
    };

    let bitcode_file_path: PathBuf = bitcode_base_path.join(bitcode_file_name);

    llvm_module.write_bitcode_to_path(&bitcode_file_path)
}
