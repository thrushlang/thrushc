extern crate anyhow;
extern crate cc;
#[macro_use]
extern crate lazy_static;
extern crate regex_lite;
extern crate semver;

use std::env;
use std::ffi::OsStr;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::Context as _;
use regex_lite::Regex;
use semver::Version;

// Environment variables that can guide compilation
//
// When adding new ones, they should also be added to main() to force a
// rebuild if they are changed.
lazy_static! {
    /// A single path to search for LLVM in (containing bin/llvm-config)
    static ref ENV_LLVM_PREFIX: String =
        format!("LLVM_SYS_{}_PREFIX", env!("CARGO_PKG_VERSION_MAJOR"));

    /// If exactly "YES", ignore the version blocklist
    static ref ENV_IGNORE_BLOCKLIST: String =
        format!("LLVM_SYS_{}_IGNORE_BLOCKLIST", env!("CARGO_PKG_VERSION_MAJOR"));

    /// If set, enforce precise correspondence between crate and binary versions.
    static ref ENV_STRICT_VERSIONING: String =
        format!("LLVM_SYS_{}_STRICT_VERSIONING", env!("CARGO_PKG_VERSION_MAJOR"));

    /// If set, do not attempt to strip irrelevant options for llvm-config --cflags
    static ref ENV_NO_CLEAN_CFLAGS: String =
        format!("LLVM_SYS_{}_NO_CLEAN_CFLAGS", env!("CARGO_PKG_VERSION_MAJOR"));

    /// If set and targeting MSVC, force the debug runtime library
    static ref ENV_USE_DEBUG_MSVCRT: String =
        format!("LLVM_SYS_{}_USE_DEBUG_MSVCRT", env!("CARGO_PKG_VERSION_MAJOR"));

    /// If set, always link against libffi
    static ref ENV_FORCE_FFI: String =
        format!("LLVM_SYS_{}_FFI_WORKAROUND", env!("CARGO_PKG_VERSION_MAJOR"));
}

lazy_static! {
    /// LLVM version used by this version of the crate.
    static ref CRATE_VERSION: Version = {
        let crate_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .expect("Crate version is somehow not valid semver");
        Version {
            major: crate_version.major / 10,
            minor: crate_version.major % 10,
            .. crate_version
        }
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

fn locate_llvm_config(path: &Path) -> Option<PathBuf> {
    if let Some(x) = llvm_compatible_binary_name(path) {
        return Some(x);
    }

    None
}

fn llvm_compatible_binary_name(prefix: &Path) -> Option<PathBuf> {
    for binary_name in llvm_config_binary_names() {
        let binary_name = prefix.join(binary_name);

        match llvm_version(&binary_name) {
            Ok(version) => {
                if is_compatible_llvm(&version) {
                    return Some(binary_name);
                }

                println!(
                    "found LLVM version {} on PATH, but need {}",
                    version, *CRATE_VERSION
                );
            }
            Err(e) => {
                if e.downcast_ref::<io::Error>()
                    .is_some_and(|e| e.kind() == ErrorKind::NotFound)
                {

                    // Looks like we failed to execute any llvm-config. Keep
                    // searching.
                } else {
                    // Some other error, probably a weird failure. Give up.
                    panic!("Failed to search PATH for llvm-config: {}", e)
                }
            }
        }
    }

    None
}

/// Return an iterator over possible names for the llvm-config binary.
fn llvm_config_binary_names() -> impl Iterator<Item = String> {
    let base_names = [
        "llvm-config".into(),
        format!("llvm-config-{}", CRATE_VERSION.major),
        format!("llvm-config{}", CRATE_VERSION.major),
        format!("llvm{}-config", CRATE_VERSION.major),
        format!(
            "llvm-config-{}.{}",
            CRATE_VERSION.major, CRATE_VERSION.minor
        ),
        format!("llvm-config{}{}", CRATE_VERSION.major, CRATE_VERSION.minor),
    ];

    // On Windows, also search for llvm-config.exe
    if target_os_is("windows") {
        IntoIterator::into_iter(base_names)
            .flat_map(|name| [format!("{}.exe", name), name])
            .collect::<Vec<_>>()
    } else {
        base_names.to_vec()
    }
    .into_iter()
}

/// Check whether the given version of LLVM is blocklisted,
/// returning `Some(reason)` if it is.
fn is_blocklisted_llvm(llvm_version: &Version) -> Option<&'static str> {
    static BLOCKLIST: &[(u64, u64, u64, &str)] = &[];

    if let Some(x) = env::var_os(&*ENV_IGNORE_BLOCKLIST) {
        if &x == "YES" {
            println!(
                "cargo:warning=ignoring blocklist entry for LLVM {}",
                llvm_version
            );
            return None;
        } else {
            println!(
                "cargo:warning={} is set but not exactly \"YES\"; blocklist is still honored",
                *ENV_IGNORE_BLOCKLIST
            );
        }
    }

    for &(major, minor, patch, reason) in BLOCKLIST.iter() {
        let bad_version = Version {
            major,
            minor,
            patch,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        };

        if &bad_version == llvm_version {
            return Some(reason);
        }
    }
    None
}

/// Check whether the given LLVM version is compatible with this version of
/// the crate.
fn is_compatible_llvm(llvm_version: &Version) -> bool {
    if let Some(reason) = is_blocklisted_llvm(llvm_version) {
        println!(
            "found LLVM {}, which is blocklisted: {}",
            llvm_version, reason
        );
        return false;
    }

    let strict =
        env::var_os(&*ENV_STRICT_VERSIONING).is_some() || cfg!(feature = "strict-versioning");
    if strict {
        llvm_version.major == CRATE_VERSION.major && llvm_version.minor == CRATE_VERSION.minor
    } else {
        llvm_version.major >= CRATE_VERSION.major
            || (llvm_version.major == CRATE_VERSION.major
                && llvm_version.minor >= CRATE_VERSION.minor)
    }
}

/// Invoke the specified binary as llvm-config.
fn llvm_config<I, S>(binary: &Path, args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    llvm_config_ex(binary, args).expect("Surprising failure from llvm-config")
}

/// Invoke the specified binary as llvm-config.
///
/// Explicit version of the `llvm_config` function that bubbles errors
/// up.
fn llvm_config_ex<I, S>(binary: &Path, args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd: Command = Command::new(binary);

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.args(args).output()?;

    let stdout = String::from_utf8(stdout).context("stdout")?;
    let stderr = String::from_utf8(stderr).context("stderr")?;
    if status.success() {
        Ok(stdout)
    } else {
        Err(anyhow::anyhow!(
            "status={status}\nstdout={}\nstderr={}",
            stdout.trim(),
            stderr.trim()
        ))
    }
}

/// Get the LLVM version using llvm-config.
fn llvm_version(binary: &Path) -> anyhow::Result<Version> {
    let version_str = llvm_config_ex(binary, ["--version"])?;

    // LLVM isn't really semver and uses version suffixes to build
    // version strings like '3.8.0svn', so limit what we try to parse
    // to only the numeric bits.
    let re = Regex::new(r"^(?P<major>\d+)\.(?P<minor>\d+)(?:\.(?P<patch>\d+))??").unwrap();
    let c = re.captures(&version_str).ok_or_else(|| {
        anyhow::anyhow!(
            "could not determine LLVM version from llvm-config. Version string: {version_str}"
        )
    })?;

    // some systems don't have a patch number but Version wants it so we just append .0 if it isn't
    // there
    let major = c.name("major").unwrap().as_str().parse().context("major")?;
    let minor = c.name("minor").unwrap().as_str().parse().context("minor")?;
    let patch = match c.name("patch") {
        None => 0,
        Some(patch) => patch.as_str().parse().context("patch")?,
    };

    Ok(Version::new(major, minor, patch))
}

/// Get the names of the dylibs required by LLVM, including the C++ standard
/// library.
fn get_system_libraries(llvm_config_path: &Path, kind: LibraryKind) -> Vec<String> {
    let link_arg = match kind {
        LibraryKind::Static => "--link-static",
        LibraryKind::Dynamic => "--link-shared",
    };

    llvm_config(llvm_config_path, ["--system-libs", link_arg])
        .split(&[' ', '\n'] as &[char])
        .filter(|s| !s.is_empty())
        .map(|flag| {
            if target_env_is("msvc") {
                // Same as --libnames, foo.lib
                flag.strip_suffix(".lib").unwrap_or_else(|| {
                    panic!(
                        "system library '{}' does not appear to be a MSVC library file",
                        flag
                    )
                })
            } else {
                if let Some(flag) = flag.strip_prefix("-l") {
                    // Linker flags style, -lfoo
                    if target_os_is("macos") {
                        // .tdb libraries are "text-based stub" files that provide lists of symbols,
                        // which refer to libraries shipped with a given system and aren't shipped
                        // as part of the corresponding SDK. They're named like the underlying
                        // library object, including the 'lib' prefix that we need to strip.
                        if let Some(flag) = flag
                            .strip_prefix("lib")
                            .and_then(|flag| flag.strip_suffix(".tbd"))
                        {
                            return flag;
                        }
                    }

                    if let Some(i) = flag.find(".so.") {
                        // On some distributions (OpenBSD, perhaps others), we get sonames
                        // like "-lz.so.7.0". Correct those by pruning the file extension
                        // and library version.
                        return &flag[..i];
                    }
                    return flag;
                }

                let maybe_lib = Path::new(flag);
                if maybe_lib.is_file() {
                    // Library on disk, likely an absolute path to a .so. We'll add its location to
                    // the library search path and specify the file as a link target.
                    println!(
                        "cargo:rustc-link-search={}",
                        maybe_lib.parent().unwrap().display()
                    );

                    // Expect a file named something like libfoo.so, or with a version libfoo.so.1.
                    // Trim everything after and including the last .so and remove the leading 'lib'
                    let soname = maybe_lib
                        .file_name()
                        .unwrap()
                        .to_str()
                        .expect("Shared library path must be a valid string");
                    let (stem, _rest) = soname
                        .rsplit_once(target_dylib_extension())
                        .expect("Shared library should be a .so file");

                    stem.strip_prefix("lib").unwrap_or_else(|| {
                        panic!("system library '{}' does not have a 'lib' prefix", soname)
                    })
                } else {
                    panic!(
                        "Unable to parse result of llvm-config --system-libs: {}",
                        flag
                    )
                }
            }
        })
        .chain(get_system_libcpp())
        .map(str::to_owned)
        .collect()
}

/// Return additional linker search paths that should be used but that are not discovered
/// by other means.
///
/// In particular, this should include only directories that are known from platform-specific
/// knowledge that aren't otherwise discovered from either `llvm-config` or a linked library
/// that includes an absolute path.
fn get_system_library_dirs(llvm_config_path: &Path) -> impl IntoIterator<Item = String> {
    let mut system_library_dirs: Vec<String> = Vec::new();

    if target_os_is("windows") || target_os_is("linux") {
        system_library_dirs.push(llvm_config_path.join("lib/").to_string_lossy().to_string());
    }

    system_library_dirs
}

fn target_dylib_extension() -> &'static str {
    if target_os_is("macos") {
        ".dylib"
    } else {
        ".so"
    }
}

/// Get the library that must be linked for C++, if any.
fn get_system_libcpp() -> Option<&'static str> {
    if let Some(libcpp) = option_env!("LLVM_SYS_LIBCPP") {
        // Use the library defined by the caller, if provided.
        Some(libcpp)
    } else if target_env_is("msvc") {
        // MSVC doesn't need an explicit one.
        None
    } else if target_os_is("macos") {
        // On OS X 10.9 and later, LLVM's libc++ is the default. On earlier
        // releases GCC's libstdc++ is default. Unfortunately we can't
        // reasonably detect which one we need (on older ones libc++ is
        // available and can be selected with -stdlib=lib++), so assume the
        // latest, at the cost of breaking the build on older OS releases
        // when LLVM was built against libstdc++.
        Some("c++")
    } else if target_os_is("freebsd") || target_os_is("openbsd") {
        Some("c++")
    } else {
        // Otherwise assume GCC's libstdc++.
        // This assumption is probably wrong on some platforms, but it can be
        // always overwritten through `LLVM_SYS_LIBCPP` variable.
        Some("stdc++")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LibraryKind {
    Static,
    Dynamic,
}

impl LibraryKind {
    pub fn string(&self) -> &'static str {
        match self {
            LibraryKind::Static => "static",
            LibraryKind::Dynamic => "dylib",
        }
    }
}

/// Get the names of libraries to link against, along with whether it is static or shared library.
fn get_link_libraries(
    llvm_config_path: &Path,
    preferences: &LinkingPreferences,
) -> (LibraryKind, Vec<String>) {
    // Using --libnames in conjunction with --libdir is particularly important
    // for MSVC when LLVM is in a path with spaces, but it is generally less of
    // a hack than parsing linker flags output from --libs and --ldflags.

    fn get_link_libraries_impl(
        llvm_config_path: &Path,
        kind: LibraryKind,
    ) -> anyhow::Result<String> {
        // Windows targets don't get dynamic support.
        // See: https://gitlab.com/taricorp/llvm-sys.rs/-/merge_requests/31#note_1306397918
        if target_env_is("msvc") && kind == LibraryKind::Dynamic {
            anyhow::bail!("Dynamic linking to LLVM is not supported on Windows");
        }

        let link_arg = match kind {
            LibraryKind::Static => "--link-static",
            LibraryKind::Dynamic => "--link-shared",
        };
        llvm_config_ex(llvm_config_path, ["--libnames", link_arg])
    }

    let LinkingPreferences {
        prefer_static,
        force,
    } = preferences;
    let one = [*prefer_static];
    let both = [*prefer_static, !*prefer_static];

    let preferences = if *force { &one[..] } else { &both[..] }
        .iter()
        .map(|is_static| {
            if *is_static {
                LibraryKind::Static
            } else {
                LibraryKind::Dynamic
            }
        });

    for kind in preferences {
        match get_link_libraries_impl(llvm_config_path, kind) {
            Ok(s) => return (kind, extract_library(&s, kind)),
            Err(err) => {
                println!(
                    "failed to get {} libraries from llvm-config: {err:?}",
                    kind.string()
                )
            }
        }
    }

    panic!("failed to get linking libraries from llvm-config");
}

fn extract_library(s: &str, kind: LibraryKind) -> Vec<String> {
    s.split(&[' ', '\n'] as &[char])
        .filter(|s| !s.is_empty())
        .map(|name| {
            // --libnames gives library filenames. Extract only the name that
            // we need to pass to the linker.
            match kind {
                LibraryKind::Static => {
                    // Match static library
                    if let Some(name) = name
                        .strip_prefix("lib")
                        .and_then(|name| name.strip_suffix(".a"))
                    {
                        // Unix (Linux/Mac)
                        // libLLVMfoo.a
                        name
                    } else if let Some(name) = name.strip_suffix(".lib") {
                        // Windows
                        // LLVMfoo.lib
                        name
                    } else {
                        panic!("'{}' does not look like a static library name", name)
                    }
                }
                LibraryKind::Dynamic => {
                    // Match shared library
                    if let Some(name) = name
                        .strip_prefix("lib")
                        .and_then(|name| name.strip_suffix(".dylib"))
                    {
                        // Mac
                        // libLLVMfoo.dylib
                        name
                    } else if let Some(name) = name
                        .strip_prefix("lib")
                        .and_then(|name| name.strip_suffix(".so"))
                    {
                        // Linux
                        // libLLVMfoo.so
                        name
                    } else if let Some(name) = IntoIterator::into_iter([".dll", ".lib"])
                        .find_map(|suffix| name.strip_suffix(suffix))
                    {
                        // Windows
                        // LLVMfoo.{dll,lib}
                        name
                    } else {
                        panic!("'{}' does not look like a shared library name", name)
                    }
                }
            }
            .to_string()
        })
        .collect::<Vec<String>>()
}

#[derive(Debug, Clone, Copy)]
struct LinkingPreferences {
    /// Prefer static linking over dynamic linking.
    prefer_static: bool,
    /// Force the use of the preferred kind of linking.
    force: bool,
}

impl LinkingPreferences {
    fn init() -> LinkingPreferences {
        let prefer_static = cfg!(feature = "prefer-static");
        let prefer_dynamic = cfg!(feature = "prefer-dynamic");
        let force_static = cfg!(feature = "force-static");
        let force_dynamic = cfg!(feature = "force-dynamic");

        // more than one preference is an error
        if [prefer_static, prefer_dynamic, force_static, force_dynamic]
            .iter()
            .filter(|&&x| x)
            .count()
            > 1
        {
            panic!(
                "Only one of the features `prefer-static`, `prefer-dynamic`, `force-static`, \
                 `force-dynamic` can be enabled at once"
            );
        }

        // if no preference is given, default to prefer static linking
        let prefer_static = prefer_static || !(prefer_dynamic || force_static || force_dynamic);

        LinkingPreferences {
            prefer_static: force_static || prefer_static,
            force: force_static || force_dynamic,
        }
    }
}

fn get_llvm_cflags(llvm_config_path: &Path) -> String {
    let output: String = llvm_config(llvm_config_path, ["--cflags"]);

    // llvm-config includes cflags from its own compilation with --cflags that
    // may not be relevant to us. In particularly annoying cases, these might
    // include flags that aren't understood by the default compiler we're
    // using. Unless requested otherwise, clean CFLAGS of options that are
    // known to be possibly-harmful.
    let no_clean = env::var_os(&*ENV_NO_CLEAN_CFLAGS).is_some();
    if no_clean || target_env_is("msvc") {
        // MSVC doesn't accept -W... options, so don't try to strip them and
        // possibly strip something that should be retained. Also do nothing if
        // the user requests it.
        return output;
    }

    output
        .split(&[' ', '\n'][..])
        .filter(|word| !word.starts_with("-W"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_llvm_debug(llvm_config_path: &Path) -> bool {
    // Has to be either Debug or Release
    llvm_config(llvm_config_path, ["--build-mode"]).contains("Debug")
}

fn main() {
    let error = |_| {
        println!("cargo:rustc-cfg=LLVM_SYS_NOT_FOUND");
        unreachable!()
    };

    let home: PathBuf = match std::env::consts::OS {
        "windows" => PathBuf::from(std::env::var("APPDATA").unwrap_or_else(error)),
        "linux" => PathBuf::from(std::env::var("HOME").unwrap_or_else(error)),
        _ => {
            println!("cargo:rustc-cfg=LLVM_SYS_NOT_FOUND");
            return;
        }
    };

    println!("cargo:rerun-if-env-changed={}", &*ENV_LLVM_PREFIX);
    if let Ok(path) = env::var(&*ENV_LLVM_PREFIX) {
        println!("cargo:rerun-if-changed={}", path);
    }

    println!("cargo:rerun-if-env-changed={}", &*ENV_IGNORE_BLOCKLIST);
    println!("cargo:rerun-if-env-changed={}", &*ENV_STRICT_VERSIONING);
    println!("cargo:rerun-if-env-changed={}", &*ENV_NO_CLEAN_CFLAGS);
    println!("cargo:rerun-if-env-changed={}", &*ENV_USE_DEBUG_MSVCRT);
    println!("cargo:rerun-if-env-changed={}", &*ENV_FORCE_FFI);

    if cfg!(feature = "no-llvm-linking") && cfg!(feature = "disable-alltargets-init") {
        return;
    }

    let llvm_path: PathBuf = home.join("thrushlang/backends/llvm/build");

    let llvm_config_path: PathBuf = match locate_llvm_config(&llvm_path.join("bin/")) {
        None => {
            println!("cargo:rustc-cfg=LLVM_SYS_NOT_FOUND");
            return;
        }
        Some(llvm_config_path) => llvm_config_path,
    };

    if !cfg!(feature = "disable-alltargets-init") {
        std::env::set_var("CFLAGS", get_llvm_cflags(&llvm_config_path));
        cc::Build::new()
            .file("wrappers/target.c")
            .compile("targetwrappers");
    }

    if cfg!(feature = "no-llvm-linking") {
        return;
    }

    let libdir: String = llvm_config(&llvm_config_path, ["--libdir"]);

    println!("cargo:config_path={}", llvm_config_path.display());
    println!("cargo:libdir={}", libdir);

    let preferences: LinkingPreferences = LinkingPreferences::init();

    println!("cargo:rustc-link-search=native={}", libdir);

    for link_search_dir in get_system_library_dirs(&llvm_path) {
        println!("cargo:rustc-link-search=native={}", link_search_dir);
    }

    let (kind, libs) = get_link_libraries(&llvm_config_path, &preferences);

    for name in libs {
        println!("cargo:rustc-link-lib={}={}", kind.string(), name);
    }

    let sys_lib_kind: LibraryKind = if cfg!(target_feature = "crt-static") {
        LibraryKind::Static
    } else {
        LibraryKind::Dynamic
    };

    for name in get_system_libraries(&llvm_config_path, kind) {
        println!("cargo:rustc-link-lib={}={}", sys_lib_kind.string(), name);
    }

    let use_debug_msvcrt = env::var_os(&*ENV_USE_DEBUG_MSVCRT).is_some();

    if target_env_is("msvc") && (use_debug_msvcrt || is_llvm_debug(&llvm_config_path)) {
        println!("cargo:rustc-link-lib=msvcrtd");
    }

    // Link libffi if the user requested this workaround.
    // See https://bitbucket.org/tari/llvm-sys.rs/issues/12/
    let force_ffi = env::var_os(&*ENV_FORCE_FFI).is_some();
    if force_ffi {
        println!("cargo:rustc-link-lib=dylib=ffi");
    }
}
