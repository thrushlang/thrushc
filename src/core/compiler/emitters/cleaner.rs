use std::path::{Path, PathBuf};

use crate::core::compiler::options::CompilerOptions;

pub fn auto_clean(options: &CompilerOptions) {
    if options.get_clean_build() {
        self::clean_build(options);
    }

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

#[inline]
fn clean_build(options: &CompilerOptions) {
    let build_path: &Path = options.get_build_dir();

    let _ = std::fs::remove_dir_all(build_path);
    let _ = std::fs::create_dir_all(build_path);
}

#[inline]
fn clean_assembler(options: &CompilerOptions) {
    let assembler_path: PathBuf = options.get_build_dir().join("emit").join("assembler");
    let _ = std::fs::remove_dir_all(&assembler_path);
}

#[inline]
fn clean_llvm_ir(options: &CompilerOptions) {
    let llvm_ir_path: PathBuf = options.get_build_dir().join("emit").join("llvm-ir");
    let _ = std::fs::remove_dir_all(&llvm_ir_path);
}

#[inline]
fn clean_llvm_bitcode(options: &CompilerOptions) {
    let llvm_bitcode_path: PathBuf = options.get_build_dir().join("emit").join("llvm-bitcode");
    let _ = std::fs::remove_dir_all(&llvm_bitcode_path);
}

#[inline]
fn clean_objects(options: &CompilerOptions) {
    let objects_path: PathBuf = options.get_build_dir().join("emit").join("obj");
    let _ = std::fs::remove_dir_all(&objects_path);
}

#[inline]
fn clean_tokens(options: &CompilerOptions) {
    let tokens_path: PathBuf = options.get_build_dir().join("emit").join("tokens");
    let _ = std::fs::remove_dir_all(&tokens_path);
}
