use std::process::Command;

#[cfg(target_os = "linux")]
fn linux_link_gccjit_if_exist() {
    let libgccjit: Result<libloading::os::unix::Library, libloading::Error> =
        unsafe { libloading::os::unix::Library::new("libgccjit") };

    if libgccjit.is_ok() {
        println!("cargo:rustc-link-lib=gccjit");
        println!("cargo:rustc-cfg=feature=\"gcc_enabled\"");
    }
}

fn main() {}

#[inline]
fn exist_clang_installation() -> bool {
    Command::new("clang").arg("-v").output().is_ok()
}

#[inline]
fn exist_llvm_linker_installation() -> bool {
    Command::new("lld").arg("-v").output().is_ok()
}
