use std::env;
use std::ffi::OsStr;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static::lazy_static! {
    static ref HOME: PathBuf = {
        let error = |_| {
            /*  logging::log(logging::LoggingType::Panic, "Unable to get user %HOME%."); */
            unreachable!()
        };

        let unsupported_os = || {
            /*logging::log(
                logging::LoggingType::Panic,
                &format!(
                    "Incompatible host operating system '{}' for compilation.",
                    env::consts::OS
                ),
            );*/
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
            /*logging::log(
                logging::LoggingType::Panic,
                &format!(
                    "Incompatible host operating system '{}' for compilation.",
                    env::consts::OS
                ),
            );*/
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
            /*logging::log(
                logging::LoggingType::Panic,
                "Unable to find 'llvm-config' for build LLD Wrapper. Use 'thorium toolchain install' to install it.",
            );*/
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
    llvm_config_ex(&*LLVM_CONFIG_PATH, arg).expect("Surprising failure from llvm-config")
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
