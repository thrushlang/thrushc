pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMPILER_ID: &str = const_format::formatcp!("thrustc version {}", COMPILER_VERSION);
pub const COMPILER_GITHUB_URL: &str = "https://github.com/thrustlang/thrustc";

pub const COMPILER_OWN_FILE_EXTENSIONS: [&str; 2] = ["thrust", "🐦"];

pub const COMPILER_HARD_OBFUSCATION_LEVEL: usize = 30;
pub const COMPILER_LOW_OBFUSCATION_LEVEL: usize = 15;
