use colored::{ColoredString, Colorize};
use std::env;
use std::ffi::OsStr;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{io::Write, process};

lazy_static::lazy_static! {
    static ref HOME: PathBuf = {
        let error = |_| {
            log(LoggingType::Panic, "Unable to get user %HOME% for build the LLVM Linker wrapper.");

            unreachable!()
        };

        let unsupported_os = || {
            log(
                LoggingType::Panic,
                &format!(
                    "Incompatible host operating system '{}' for compilation of the LLVM Linker wrapper.",
                    env::consts::OS
                ),
            );

            unreachable!()
        };

        match env::consts::OS {
            "windows" => PathBuf::from(env::var("APPDATA").unwrap_or_else(error)),
            "linux" => PathBuf::from(env::var("HOME").unwrap_or_else(error)),
            _ => {
                unsupported_os();
                unreachable!();
            }
        }
    };

    static ref EXECUTABLE_EXTENSION: &'static str = {
        let unsupported_os = || {
            log(
                LoggingType::Panic,
                &format!(
                    "Incompatible host operating system '{}' for compilation of the LLVM Linker wrapper.",
                    env::consts::OS
                ),
            );

            unreachable!()
        };

        match env::consts::OS {
            "windows" => ".exe",
            "linux" => "",
            _ => {
                unsupported_os();
                unreachable!();
            }
        }
    };

    static ref LLVM_CONFIG_PATH: PathBuf = {
        let system_executables_extension: &str = &EXECUTABLE_EXTENSION;
        let llvm_config_path = HOME.join(format!("thrushlang/backends/llvm/build/bin/llvm-config{}", system_executables_extension));

        if !llvm_config_path.exists() {
            log(
                LoggingType::Panic,
                "Unable to find 'llvm-config' for build LLVM Linker wrapper. Use 'thorium toolchain llvm install' to install it.",
            );

            unreachable!()
        }

        llvm_config_path
    };
}

fn target_env_is(name: &str) -> bool {
    match env::var_os("CARGO_CFG_TARGET_ENV") {
        Some(s) => s == name,
        None => false,
    }
}

fn target_os_is(name: &str) -> bool {
    match env::var_os("CARGO_CFG_TARGET_OS") {
        Some(s) => s == name,
        None => false,
    }
}

fn llvm_config(arg: &str) -> String {
    llvm_config_ex(&*LLVM_CONFIG_PATH, arg).unwrap_or_else(|_| {
        log(LoggingType::Panic, "Unable to invoke llvm-config.");
        unreachable!()
    })
}

fn llvm_config_ex<S: AsRef<OsStr>>(binary: S, arg: &str) -> io::Result<String> {
    Command::new(binary)
        .arg(arg)
        .arg("--link-static")
        .output()
        .and_then(|output| {
            if output.stdout.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "llvm-config returned empty output",
                ))
            } else {
                Ok(String::from_utf8(output.stdout)
                    .expect("Output from llvm-config was not valid UTF-8"))
            }
        })
}

fn get_system_libraries() -> Vec<String> {
    llvm_config("--system-libs")
        .split(&[' ', '\n'] as &[char])
        .filter(|s| !s.is_empty())
        .filter(|s| !s.starts_with("/"))
        .map(|flag| {
            if target_env_is("msvc") {
                assert!(
                    flag.ends_with(".lib"),
                    "system library {:?} does not appear to be a MSVC library file",
                    flag
                );
                &flag[..flag.len() - 4]
            } else {
                if flag.starts_with("-l") {
                    if target_os_is("macos") && flag.starts_with("-llib") && flag.ends_with(".tbd")
                    {
                        return flag[5..flag.len() - 4].to_owned();
                    }
                    return flag[2..].to_owned();
                }

                let maybe_lib = Path::new(&flag);
                if maybe_lib.is_file() {
                    println!(
                        "cargo:rustc-link-search={}",
                        maybe_lib.parent().unwrap().display()
                    );

                    let soname = maybe_lib
                        .file_name()
                        .unwrap()
                        .to_str()
                        .expect("Shared library path must be a valid string");
                    let stem = soname
                        .rsplit_once(".so")
                        .expect("Shared library should be a .so file")
                        .0;

                    stem.trim_start_matches("lib")
                } else {
                    panic!(
                        "Unable to parse result of llvm-config --system-libs: was {:?}",
                        flag
                    )
                }
            }
            .to_owned()
        })
        .chain(get_system_libcpp().map(str::to_owned))
        .collect::<Vec<String>>()
}

fn get_system_libcpp() -> Option<&'static str> {
    if target_env_is("msvc") {
        None
    } else {
        Some("stdc++")
    }
}

fn get_link_libraries() -> Vec<String> {
    llvm_config("--libnames")
        .split(&[' ', '\n'] as &[char])
        .filter(|s| !s.is_empty())
        .map(|name| {
            if target_env_is("msvc") {
                assert!(
                    name.ends_with(".lib"),
                    "library name {:?} does not appear to be a MSVC library file",
                    name
                );
                &name[..name.len() - 4]
            } else {
                assert!(
                    name.starts_with("lib") && name.ends_with(".a"),
                    "library name {:?} does not appear to be a static library",
                    name
                );
                &name[3..name.len() - 2]
            }
        })
        .map(str::to_owned)
        .collect::<Vec<String>>()
}

fn get_llvm_cxxflags() -> String {
    let output = llvm_config("--cxxflags");

    let no_clean = env::var_os(format!(
        "LLVM_SYS_{}_NO_CLEAN_CFLAGS",
        env!("CARGO_PKG_VERSION_MAJOR")
    ))
    .is_some();
    if no_clean || target_env_is("msvc") {
        return output;
    }

    llvm_config("--cxxflags")
        .split(&[' ', '\n'][..])
        .filter(|word| !word.starts_with("-W"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    unsafe { env::set_var("CXXFLAGS", get_llvm_cxxflags()) };

    let mut build = cc::Build::new();

    build.cpp(true).file("wrapper/library.cpp");

    if build.get_compiler().is_like_msvc() {
        build.flag("/std:c++17");
    } else {
        build.flag("-std=c++17");
    }

    build.compile("lldwrapper");

    println!("cargo:rerun-if-changed=wrapper/library.cpp");

    let libdir: String = llvm_config("--libdir");

    println!("cargo:config_path={}", LLVM_CONFIG_PATH.display());
    println!("cargo:libdir={}", libdir);

    println!("cargo:rustc-link-search=native={}", libdir);

    let blacklist: Vec<&'static str> = vec!["LLVMLineEditor"];

    for name in get_link_libraries()
        .iter()
        .filter(|n| !blacklist.iter().any(|blacklisted| n.contains(*blacklisted)))
    {
        println!("cargo:rustc-link-lib=static={}", name);
    }

    for name in get_system_libraries() {
        println!("cargo:rustc-link-lib=dylib={}", name);
    }

    if cfg!(target_env = "msvc") {
        println!("cargo:rustc-link-lib=msvcrtd");
    }

    println!("cargo:rustc-link-lib=static=lldWasm");
    println!("cargo:rustc-link-lib=static=lldCOFF");
    println!("cargo:rustc-link-lib=static=lldCommon");
    println!("cargo:rustc-link-lib=static=lldELF");
    println!("cargo:rustc-link-lib=static=lldMachO");
    println!("cargo:rustc-link-lib=static=lldMinGW");
    println!("cargo:rustc-link-lib=static=lldWasm");

    if cfg!(not(target_os = "windows")) {
        println!("cargo:rustc-link-lib=dylib=ffi");
    }
}

#[derive(Debug)]
pub enum OutputIn {
    Stdout,
    Stderr,
}

#[derive(Debug, PartialEq)]
pub enum LoggingType {
    Error,
    Warning,
    Panic,
}

impl LoggingType {
    pub fn to_styled(&self) -> ColoredString {
        match self {
            LoggingType::Error => "ERROR".bright_red().bold(),
            LoggingType::Warning => "WARN".yellow().bold(),
            LoggingType::Panic => "PANIC".bold().bright_red().underline(),
        }
    }

    pub fn is_panic(&self) -> bool {
        matches!(self, LoggingType::Panic)
    }

    pub fn is_err(&self) -> bool {
        matches!(self, LoggingType::Error)
    }

    pub fn is_warn(&self) -> bool {
        matches!(self, LoggingType::Warning)
    }

    pub fn text_with_color(&self, msg: &str) -> ColoredString {
        match self {
            LoggingType::Error => msg.bright_red().bold(),
            LoggingType::Warning => msg.yellow().bold(),
            LoggingType::Panic => msg.bright_red().underline(),
        }
    }
}

pub fn log(ltype: LoggingType, msg: &str) {
    if ltype.is_panic() {
        io::stderr()
            .write_all(format!("{} {}\n  ", ltype.to_styled(), msg.bold()).as_bytes())
            .unwrap();

        process::exit(1);
    }

    if ltype.is_err() {
        io::stderr()
            .write_all(format!("{} {}\n  ", ltype.to_styled(), msg.bold()).as_bytes())
            .unwrap();

        return;
    }

    if ltype.is_warn() {
        io::stderr()
            .write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes())
            .unwrap();

        return;
    }

    io::stdout()
        .write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes())
        .unwrap();
}

pub fn write(output_in: OutputIn, text: &str) {
    match output_in {
        OutputIn::Stdout => io::stdout().write_all(text.as_bytes()).unwrap_or(()),
        OutputIn::Stderr => io::stderr().write_all(text.as_bytes()).unwrap_or(()),
    };
}
