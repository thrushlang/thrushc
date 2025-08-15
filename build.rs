fn main() {
    if cfg!(target_os = "linux") {
        println!("cargo:warning=yeeee");
        println!("cargo:rustc-linker=clang");
        println!("cargo:rustc-link-arg=-fuse-ld=lld");
    }
}
