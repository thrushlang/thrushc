#[inline]
pub fn get_default_dynamic_c_runtime() -> &'static str {
    match std::env::consts::OS {
        "linux" => "libc.so",
        "windows" => "msvcrt.dll",
        _ => "libc.so",
    }
}
