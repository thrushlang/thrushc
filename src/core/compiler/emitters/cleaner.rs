use std::path::PathBuf;

use crate::core::compiler::options::CompilerOptions;

pub fn auto_clean(options: &CompilerOptions) {
    if options.get_clean_assembler() {
        self::clean_assembler(options);
    }

    if options.get_clean_llvm_ir() {
        self::clean_llvm_ir(options);
    }

    if options.get_clean_llvm_bitcode() {
        self::clean_llvm_bitcode(options);
    }

    if options.get_clean_object() {
        self::clean_objects(options);
    }

    if options.get_clean_tokens() {
        self::clean_tokens(options);
    }
}

fn clean_assembler(options: &CompilerOptions) {
    let assembler_path: PathBuf = options.get_build_dir().join("emit").join("assembler");
    let _ = std::fs::remove_dir_all(&assembler_path);
}

fn clean_llvm_ir(options: &CompilerOptions) {
    let llvm_ir_path: PathBuf = options.get_build_dir().join("emit").join("llvm-ir");
    let _ = std::fs::remove_dir_all(&llvm_ir_path);
}

fn clean_llvm_bitcode(options: &CompilerOptions) {
    let llvm_bitcode_path: PathBuf = options.get_build_dir().join("emit").join("llvm-bitcode");
    let _ = std::fs::remove_dir_all(&llvm_bitcode_path);
}

fn clean_objects(options: &CompilerOptions) {
    let objects_path: PathBuf = options.get_build_dir().join("emit").join("obj");
    let _ = std::fs::remove_dir_all(&objects_path);
}

fn clean_tokens(options: &CompilerOptions) {
    let tokens_path: PathBuf = options.get_build_dir().join("emit").join("tokens");
    let _ = std::fs::remove_dir_all(&tokens_path);
}
