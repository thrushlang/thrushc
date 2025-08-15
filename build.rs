use std::process::Command;

fn main() {
    if cfg!(target_os = "linux")
        && self::exist_clang_installation()
        && self::exist_llvm_linker_installation()
    {
        println!("cargo:rustc-linker=clang");
        println!("cargo:rustc-link-arg=-fuse-ld=lld");
    }
}

#[inline]
fn exist_clang_installation() -> bool {
    Command::new("clang").arg("-v").output().is_ok()
}

#[inline]
fn exist_llvm_linker_installation() -> bool {
    Command::new("lld").arg("-v").output().is_ok()
}
