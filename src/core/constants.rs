pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMPILER_ID: &str = const_format::formatcp!("thrushc version {}", COMPILER_VERSION);
pub const COMPILER_GITHUB_URL: &str = "https://github.com/thrushlang/thrushc";

pub const COMPILER_HARD_OBFUSCATION_LEVEL: usize = 30;
pub const COMPILER_LOW_OBFUSCATION_LEVEL: usize = 15;
