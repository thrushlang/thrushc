use std::{
    borrow::Cow,
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use colored::{ColoredString, Colorize};
use isahc::{
    Body, HttpClient, ReadResponseExt, Response,
    config::{Configurable, RedirectPolicy},
};

use serde_json::{Map, Value};

use std::{
    io::{self, Write},
    process,
};

mod utils;

const GITHUB_RELEASES: &str = "https://api.github.com/repos/thrushlang/toolchains/releases";

#[derive(Debug, PartialEq)]
enum LoggingType {
    Error,
    Panic,
    Log,
}

impl LoggingType {
    #[inline]
    fn to_styled(&self) -> ColoredString {
        match self {
            LoggingType::Error => "ERROR".bright_red().bold(),
            LoggingType::Panic => "PANIC".bold().bright_red().underline(),
            LoggingType::Log => "LOG".custom_color((141, 141, 142)).bold(),
        }
    }
}

impl LoggingType {
    #[inline]
    fn is_panic(&self) -> bool {
        matches!(self, LoggingType::Panic)
    }

    #[inline]
    fn is_err(&self) -> bool {
        matches!(self, LoggingType::Error)
    }
}

fn log(ltype: LoggingType, msg: &str) {
    if ltype.is_panic() {
        let _ = io::stderr().write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes());
        process::exit(1);
    }

    if ltype.is_err() {
        let _ = io::stderr().write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes());
        return;
    }

    let _ = io::stdout().write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes());
}

#[derive(Debug)]
pub struct Builder {
    build_path: PathBuf,
}

impl Builder {
    pub fn new(home: PathBuf) -> Self {
        let build_path: PathBuf = home.join("thrushlang/backends/llvm/build");
        Self { build_path }
    }
}

impl Builder {
    pub fn install(&self) {
        self.reset_build_path();

        if let Err(err) = self.install_llvm_c_api() {
            self::log(LoggingType::Panic, &err);
        }

        log(LoggingType::Log, "LLVM C API installed.\n");

        if let Ok(thrushc_home) = self.get_thrushc_home() {
            self.reset_embedded_path(thrushc_home.join("embedded"));

            if let Err(error) = self.install_embedded_compilers() {
                self::log(LoggingType::Panic, &error);
            }

            log(LoggingType::Log, "Embedded compilers installed.\n");
        } else {
            log(LoggingType::Panic, "Unable to get 'thrushc' project.\n");
        }

        self::log(
            LoggingType::Log,
            "The Thrush Compiler dependencies are succesfully installed.\n",
        );
    }

    fn install_embedded_compilers(&self) -> Result<(), String> {
        log(LoggingType::Log, "Installing embedded compilers...\n");

        let mut releases: Response<Body> = isahc::get(GITHUB_RELEASES)
            .map_err(|_| "Unable to get releases of embedded compilers.")?;

        let releases_json: Value = releases
            .json::<Value>()
            .map_err(|_| "Unable to convert github releases to json format.")?;

        let raw_releases_json: &Vec<Value> = releases_json
            .as_array()
            .ok_or("Unable to convert github releases json format to array.")?;

        let tag_to_find: &'static str = if cfg!(target_os = "windows") {
            "CLANG-WINDOWS"
        } else {
            "CLANG-LINUX"
        };

        let tag_name_pattern_to_replace: &'static str = if cfg!(target_os = "windows") {
            "CLANG-WINDOWS-v"
        } else {
            "CLANG-LINUX-v"
        };

        let latest_embedded_compiler: Option<&Map<String, Value>> = raw_releases_json
            .iter()
            .filter_map(|release| {
                let release_obj: &Map<String, Value> = release.as_object()?;
                let tag_name: &str = release_obj.get("tag_name")?.as_str()?;

                if tag_name.starts_with(tag_to_find) {
                    Some(release_obj)
                } else {
                    None
                }
            })
            .max_by(|left, right| {
                let extract_version_sum = |item: &&Map<String, Value>| -> u8 {
                    if let Some(tag_name) = item.get("tag_name").and_then(|v| v.as_str()) {
                        if tag_name.starts_with(tag_name_pattern_to_replace) {
                            let version_string = tag_name.replace(tag_name_pattern_to_replace, "");
                            return version_string
                                .split('.')
                                .filter_map(|x| x.parse::<u8>().ok())
                                .sum();
                        }
                    }
                    0
                };

                let left_version: u8 = extract_version_sum(left);
                let right_version: u8 = extract_version_sum(right);

                left_version.cmp(&right_version)
            });

        let embedded_compiler: &Map<String, Value> = latest_embedded_compiler
            .ok_or("No embedded compiler was found for the current operating system.")?;

        let assets = embedded_compiler
            .get("assets")
            .and_then(|v| v.as_array())
            .ok_or("Unable to get assets from the latest embedded compiler.")?;

        let mut links: Vec<&str> = Vec::with_capacity(10);
        let mut names: Vec<&str> = Vec::with_capacity(10);

        for asset in assets {
            if let Some(asset_obj) = asset.as_object() {
                if let Some(url) = asset_obj
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())
                {
                    if let Some(name) = asset_obj.get("name").and_then(|v| v.as_str()) {
                        links.push(url);
                        names.push(name);
                    }
                }
            }
        }

        if links.is_empty() || names.is_empty() {
            return Err("No valid assets found in the latest embedded compiler release.".into());
        }

        self.download_and_install_embedded_compilers(links, names)
    }

    fn install_llvm_c_api(&self) -> Result<(), String> {
        log(LoggingType::Log, "Installing LLVM C API...\n");

        let mut llvm_c_apis: Response<Body> = isahc::get(GITHUB_RELEASES)
            .map_err(|_| "Could not make request to GitHub releases API.".to_string())?;

        let llvm_c_apis_json: Value = llvm_c_apis
            .json::<Value>()
            .map_err(|_| "Could not parse response as JSON.".to_string())?;

        let all_llvm_c_apis: &Vec<Value> = llvm_c_apis_json
            .as_array()
            .ok_or("Expected array in JSON response.")?;

        let tag_name_to_find: &'static str = if cfg!(target_os = "windows") {
            "LLVM-C-API-WINDOWS"
        } else {
            "LLVM-C-API-LINUX"
        };

        let tag_name_pattern_to_replace: &'static str = if cfg!(target_os = "windows") {
            "LLVM-C-API-WINDOWS-v"
        } else {
            "LLVM-C-API-LINUX-v"
        };

        let latest_llvm_c_api: Option<&Value> = all_llvm_c_apis
            .iter()
            .filter_map(|llvm_c_api| {
                let obj = llvm_c_api.as_object()?;
                let tag_name = obj.get("tag_name")?.as_str()?;

                if tag_name.starts_with(tag_name_to_find) {
                    Some(llvm_c_api)
                } else {
                    None
                }
            })
            .max_by(|left, right| {
                let extract_version = |item: &Value| -> u8 {
                    if let Some(obj) = item.as_object() {
                        if let Some(tag_name) = obj.get("tag_name").and_then(|v| v.as_str()) {
                            let version_string = tag_name.replace(tag_name_pattern_to_replace, "");
                            return version_string
                                .split('.')
                                .filter_map(|x| x.parse::<u8>().ok())
                                .sum();
                        }
                    }
                    0
                };

                let left_version: u8 = extract_version(left);
                let right_version: u8 = extract_version(right);

                left_version.cmp(&right_version)
            });

        if let Some(latest_llvm_c_api) = latest_llvm_c_api {
            if let Some(llvm_c_api) = latest_llvm_c_api.as_object() {
                if let Some(assets) = llvm_c_api.get("assets").and_then(|v| v.as_array()) {
                    let mut links: Vec<&str> = Vec::with_capacity(10);
                    let mut names: Vec<&str> = Vec::with_capacity(10);

                    for asset in assets {
                        if let Some(asset_obj) = asset.as_object() {
                            if let Some(download_url) = asset_obj
                                .get("browser_download_url")
                                .and_then(|v| v.as_str())
                            {
                                if let Some(name) = asset_obj.get("name").and_then(|v| v.as_str()) {
                                    links.push(download_url);
                                    names.push(name);
                                }
                            }
                        }
                    }

                    if !links.is_empty() && !names.is_empty() {
                        return self.download_and_install_llvm_libraries(links, names);
                    }
                }
            }
        }

        Err("No compatible LLVM-C API releases found.".to_string())
    }

    fn download_and_install_embedded_compilers(
        &self,
        links: Vec<&str>,
        names: Vec<&str>,
    ) -> Result<(), String> {
        if links.len() != names.len() {
            return Err("Links and names have different lengths.".to_string());
        }

        let client: HttpClient = HttpClient::builder()
            .redirect_policy(RedirectPolicy::Follow)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        for (link, name) in links.iter().zip(names.iter()) {
            let full_path: PathBuf = if cfg!(target_os = "linux") {
                let thrushc_path: PathBuf = self.get_thrushc_home()?;

                let base_path: PathBuf = if name.contains("clang") {
                    thrushc_path.join("embedded/compilers/linux/clang")
                } else {
                    thrushc_path.join("embedded/compilers/linux")
                };

                let _ = fs::create_dir_all(&base_path);

                base_path.join(name)
            } else {
                let thrushc_path: PathBuf = self.get_thrushc_home()?;

                let base_path: PathBuf = if name.contains("clang") {
                    thrushc_path.join("embedded/compilers/windows/clang")
                } else {
                    thrushc_path.join("embedded/compilers/windows")
                };

                let _ = fs::create_dir_all(&base_path);

                base_path.join(name)
            };

            if full_path.exists() {
                continue;
            }

            let mut response: Response<Body> = client
                .get(link.to_string())
                .map_err(|e| format!("Failed to download {}: {}", link, e))?;

            if !response.status().is_success() {
                return Err(format!(
                    "Failed to download {}: HTTP {}",
                    link,
                    response.status()
                ));
            }

            let bytes = response
                .bytes()
                .map_err(|e| format!("Failed to read response for {}: {}", link, e))?;

            let mut file = File::create(&full_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", full_path, e))?;

            file.write_all(&bytes)
                .map_err(|e| format!("Failed to write to file {:?}: {}", full_path, e))?;
        }

        Ok(())
    }

    fn download_and_install_llvm_libraries(
        &self,
        links: Vec<&str>,
        names: Vec<&str>,
    ) -> Result<(), String> {
        if links.len() != names.len() {
            return Err("Links and names have different lengths.".to_string());
        }

        let client: HttpClient = HttpClient::builder()
            .redirect_policy(RedirectPolicy::Follow)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        for (link, name) in links.iter().zip(names.iter()) {
            let full_path: PathBuf = self.build_path.join(name);

            let mut response: Response<Body> = client
                .get(link.to_string())
                .map_err(|e| format!("Failed to download {}: {}", link, e))?;

            if full_path.exists() {
                continue;
            }

            if !response.status().is_success() {
                return Err(format!(
                    "Failed to download {}: HTTP {}",
                    link,
                    response.status()
                ));
            }

            let bytes = response
                .bytes()
                .map_err(|e| format!("Failed to read response for {}: {}", link, e))?;

            let mut file = File::create(&full_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", full_path, e))?;

            file.write_all(&bytes)
                .map_err(|e| format!("Failed to write to file {:?}: {}", full_path, e))?;
        }

        for name in names.iter() {
            let full_path: PathBuf = self.build_path.join(name);

            if full_path
                .extension()
                .is_some_and(|extension| extension == "xz" || extension == "zip")
            {
                self.decompress_file(&full_path)?;
                let _ = fs::remove_file(&full_path);
            }
        }

        if cfg!(target_os = "linux") {
            self.make_executable(&self.build_path.join("bin/llvm-config"))?;
        }

        Ok(())
    }

    fn get_thrushc_home(&self) -> Result<PathBuf, String> {
        if let Ok(current_dir) = env::current_dir() {
            for ancestor in current_dir.ancestors() {
                if let Some(file_name_os_str) = ancestor.file_name() {
                    if let Some(file_name_str) = file_name_os_str.to_str() {
                        if file_name_str == "thrushc" {
                            return Ok(ancestor.to_path_buf());
                        }
                    }
                }
            }
        }

        Err("No 'thrushc' home found.".into())
    }

    fn decompress_file(&self, file_path: &Path) -> Result<(), String> {
        let mut tar_command: Command = Command::new("tar");

        tar_command
            .arg("-xf")
            .arg(file_path)
            .arg("-C")
            .arg(&self.build_path);

        if cfg!(target_os = "windows") {
            tar_command.arg("--strip-components=1");
        }

        if let Ok(tar_output) = tar_command.output() {
            let stderr: Cow<'_, str> = String::from_utf8_lossy(&tar_output.stderr);

            if !tar_output.status.success() {
                self::log(LoggingType::Error, &stderr);
            }

            return Ok(());
        }

        Err(format!(
            "'{}' could not be decompressed.",
            file_path.display()
        ))
    }

    fn make_executable(&self, file_path: &Path) -> Result<(), String> {
        let mut chmod_command: Command = Command::new("chmod");

        chmod_command.arg("+x").arg(file_path);

        if let Ok(chmod_output) = chmod_command.output() {
            let stderr: Cow<'_, str> = String::from_utf8_lossy(&chmod_output.stderr);

            if !chmod_output.status.success() {
                self::log(LoggingType::Error, &stderr);
            }

            return Ok(());
        }

        Err(format!(
            "'{}' could not be made executable.",
            file_path.display()
        ))
    }

    fn reset_build_path(&self) {
        let _ = fs::remove_dir_all(&self.build_path);
        let _ = fs::create_dir_all(&self.build_path);
    }

    fn reset_embedded_path(&self, embedded_path: PathBuf) {
        let _ = fs::remove_dir_all(&embedded_path);
        let _ = fs::create_dir_all(&embedded_path);
    }
}

fn main() {
    unsafe { env::set_var("CARGO_TERM_VERBOSE", "true") };

    #[cfg(target_os = "windows")]
    {
        colored::control::set_virtual_terminal(true);
        colored::control::set_override(true);
    }

    #[cfg(target_os = "linux")]
    {
        colored::control::set_override(true);
    }

    self::log(
        LoggingType::Log,
        "Starting to build dependencies for The Thrush Compiler...\n\n",
    );

    self::log(LoggingType::Log, "Checking requirements...\n");

    if !utils::tar_is_available() {
        self::log(LoggingType::Panic, "tar is not installed.\n");
        return;
    }

    self::log(LoggingType::Log, "Requirements are ok.\n\n");

    let home_path: String = if cfg!(target_os = "linux") {
        env::var("HOME").unwrap_or_else(|_| {
            self::log(LoggingType::Panic, "Missing $HOME environment variable.\n");
            unreachable!()
        })
    } else {
        env::var("APPDATA").unwrap_or_else(|_| {
            self::log(
                LoggingType::Panic,
                "Missing $APPDATA environment variable.\n",
            );
            unreachable!()
        })
    };

    Builder::new(PathBuf::from(home_path)).install();
}
