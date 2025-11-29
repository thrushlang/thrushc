use std::process::Command;

#[cfg(target_family = "unix")]
fn linux_link_gccjit_if_exist() {
    let libgccjit: Result<libloading::os::unix::Library, libloading::Error> =
        unsafe { libloading::os::unix::Library::new("libgccjit") };

    if libgccjit.is_ok() {
        println!("cargo:rustc-link-lib=gccjit");
        println!("cargo:rustc-cfg=feature=\"gcc_enabled\"");
    }
}

fn main() {
    #[cfg(target_family = "unix")]
    {
        if self::exist_clang_installation() && self::exist_llvm_linker_installation() {
            println!("cargo:rustc-linker=clang");
            println!("cargo:rustc-link-arg=-fuse-ld=lld");
        }

        self::linux_link_gccjit_if_exist();
    }
}

#[cfg(target_family = "unix")]
#[inline]
fn exist_clang_installation() -> bool {
    Command::new("clang").arg("-v").output().is_ok()
}

#[cfg(target_family = "unix")]
#[inline]
fn exist_llvm_linker_installation() -> bool {
    Command::new("lld").arg("-v").output().is_ok()
}
