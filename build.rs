fn main() {
    #[cfg(all(target_family = "unix", feature = "gcc_backend_dynamic"))]
    {
        let libgccjit: Result<libloading::os::unix::Library, libloading::Error> =
            unsafe { libloading::os::unix::Library::new("libgccjit") };

        if libgccjit.is_ok() {
            println!("cargo:rustc-link-lib=gccjit");
            println!("cargo:rustc-cfg=feature=\"gcc_enabled\"");
        }
    }

    if self::exist_clang_installation() && self::exist_llvm_linker_installation() {
        println!("cargo:rustc-linker=clang");
        println!("cargo:rustc-link-arg=-fuse-ld=lld");
    }
}

#[inline]
fn exist_clang_installation() -> bool {
    std::process::Command::new("clang")
        .arg("-v")
        .output()
        .is_ok()
}

#[inline]
fn exist_llvm_linker_installation() -> bool {
    std::process::Command::new("lld").arg("-v").output().is_ok()
}
