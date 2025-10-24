#[cfg(target_os = "linux")]
pub static LINUX_X86_64_CLANG: &[u8] =
    include_bytes!("../../embedded/compilers/linux/clang/clang-linux-x86_64.tar.xz");

#[cfg(target_os = "linux")]
pub static LINUX_X86_64_CLANG_MANIFEST: &str =
    include_str!("../../embedded/compilers/linux/clang/clang-manifest.json");

#[cfg(target_os = "windows")]
pub static WINDOWS_X86_64_CLANG: &[u8] =
    include_bytes!("../../embedded/compilers/windows/clang/clang-windows-x86_64.zip");

#[cfg(target_os = "windows")]
pub static WINDOWS_X86_64_CLANG_MANIFEST: &str =
    include_str!("../../embedded/compilers/windows/clang/clang-manifest.json");
