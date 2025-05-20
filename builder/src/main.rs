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

const THRUSH_LLVM_C_APIS_GITHUB_URL: &str =
    "https://api.github.com/repos/thrushlang/toolchains/releases";

#[derive(Debug, PartialEq)]
enum LoggingType {
    Error,
    Panic,
    Log,
}

impl LoggingType {
    fn to_styled(&self) -> ColoredString {
        match self {
            LoggingType::Error => "ERROR".bright_red().bold(),
            LoggingType::Panic => "PANIC".bold().bright_red().underline(),
            LoggingType::Log => "LOG".custom_color((141, 141, 142)).bold(),
        }
    }

    fn is_panic(&self) -> bool {
        matches!(self, LoggingType::Panic)
    }

    fn is_err(&self) -> bool {
        matches!(self, LoggingType::Error)
    }
}

fn log(ltype: LoggingType, msg: &str) {
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

    io::stdout()
        .write_all(format!("{} {}", ltype.to_styled(), msg.bold()).as_bytes())
        .unwrap();
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

    pub fn install(&self) {
        if self.build_path.exists() {
            self::log(
                LoggingType::Log,
                "The Thrush Compiler dependencies are already exists.\n",
            );

            return;
        }

        let _ = fs::create_dir_all(&self.build_path);

        if let Err(err) = self.install_llvm_c_api() {
            self::log(LoggingType::Panic, &err);
        }

        self::log(
            LoggingType::Log,
            "\nThe Thrush Compiler dependencies are succesfully installed.\n",
        );
    }

    fn install_llvm_c_api(&self) -> Result<(), String> {
        if let Ok(mut llvm_c_apis) = isahc::get(THRUSH_LLVM_C_APIS_GITHUB_URL) {
            if let Ok(llvm_c_apis_json) = llvm_c_apis.json::<Value>() {
                let all_llvm_c_apis: &Vec<Value> =
                    llvm_c_apis_json.as_array().expect("Expected 'array' type.");

                let latest_llvm_c_api: Option<&Value> = all_llvm_c_apis
                    .iter()
                    .filter(|llvm_c_api| {
                        let llvm_c_api = llvm_c_api.as_object().unwrap();

                        let tag_name_to_find: &'static str = if cfg!(target_os = "windows") {
                            "LLVM-C-API-WINDOWS"
                        } else {
                            "LLVM-C-API-LINUX"
                        };

                        llvm_c_api
                            .get("tag_name")
                            .expect("Expected to get 'tag_name' field.")
                            .as_str()
                            .expect("Expected '&str' type.")
                            .starts_with(tag_name_to_find)
                    })
                    .max_by(|left, right| {
                        let left: &Map<String, Value> =
                            left.as_object().expect("Expected 'object' type.");

                        let right: &Map<String, Value> =
                            right.as_object().expect("Expected 'object' type.");

                        let tag_name_pattern_to_replace: &'static str =
                            if cfg!(target_os = "windows") {
                                "LLVM-C-API-WINDOWS-v"
                            } else {
                                "LLVM-C-API-LINUX-v"
                            };

                        let left_tag_name: String = left
                            .get("tag_name")
                            .expect("Expected to get 'tag_name' field.")
                            .as_str()
                            .expect("Expected '&str' type.")
                            .replace(tag_name_pattern_to_replace, "");

                        let right_tag_name: String = right
                            .get("tag_name")
                            .expect("Expected to get 'tag_name' field.")
                            .as_str()
                            .expect("Expected '&str' type.")
                            .replace(tag_name_pattern_to_replace, "");

                        let left_version: u8 = left_tag_name
                            .split('.')
                            .filter_map(|x| x.parse::<u8>().ok())
                            .collect::<Vec<_>>()
                            .iter()
                            .sum();

                        let right_version: u8 = right_tag_name
                            .split('.')
                            .filter_map(|x| x.parse::<u8>().ok())
                            .collect::<Vec<_>>()
                            .iter()
                            .sum();

                        left_version.cmp(&right_version)
                    });

                if latest_llvm_c_api.is_none() {
                    return Err("No windows LLVM-C APIs found.".into());
                }

                if let Some(latest_llvm_c_api) = latest_llvm_c_api {
                    let llvm_c_api: &Map<String, Value> = latest_llvm_c_api
                        .as_object()
                        .expect("Expected 'object' type.");

                    let assets: &Vec<Value> = llvm_c_api
                        .get("assets")
                        .expect("Expected to get 'assets' field.")
                        .as_array()
                        .expect("Expected 'array' type");

                    let links: Vec<&str> = assets
                        .iter()
                        .map(|asset| {
                            let asset: &Map<String, Value> =
                                asset.as_object().expect("Expected 'object' type.");

                            asset
                                .get("browser_download_url")
                                .expect("Expecteed to get 'browser_download_url' field.")
                                .as_str()
                                .expect("Expected '&str' type.")
                        })
                        .collect();

                    let names: Vec<&str> = assets
                        .iter()
                        .map(|asset| {
                            let asset: &Map<String, Value> =
                                asset.as_object().expect("Expected 'object' type.");

                            asset
                                .get("name")
                                .expect("Expecteed to get 'name' field.")
                                .as_str()
                                .expect("Expected '&str' type.")
                        })
                        .collect();

                    self.download_and_install_llvm_libraries(links, names)?;

                    return Ok(());
                }
            }

            return Err(
                "The request 'https://api.github.com/repos/thrushlang/toolchains/releases' could not be transformed into json.".into(),
            );
        }

        Err(
            "A request could not be made to 'https://api.github.com/repos/thrushlang/toolchains/releases'.".into(),
        )
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

            if full_path.exists() {
                if name == &"llvm-config" {
                    if cfg!(target_os = "linux") {
                        self.make_executable(&full_path)?;
                    }

                    let bin_dir: PathBuf = self.build_path.join("bin");

                    if !bin_dir.exists() {
                        fs::create_dir_all(&bin_dir).map_err(|e| {
                            format!("Failed to create bin directory {:?}: {}", bin_dir, e)
                        })?;
                    }

                    let output_path: PathBuf = bin_dir.join(name);

                    if full_path.exists() && !output_path.exists() {
                        fs::copy(&full_path, &output_path).map_err(|e| {
                            format!("Failed to copy llvm-config to {:?}: {}", output_path, e)
                        })?;

                        let _ = fs::remove_file(&full_path);
                    }
                }

                if full_path
                    .extension()
                    .is_some_and(|extension| extension == "xz")
                {
                    self.decompress_file(&full_path)?;
                    let _ = fs::remove_file(&full_path);
                }
            }
        }

        Ok(())
    }

    fn decompress_file(&self, file_path: &Path) -> Result<(), String> {
        let mut tar_command: Command = Command::new("tar");

        tar_command
            .arg("-xf")
            .arg(file_path)
            .arg("-C")
            .arg(&self.build_path);

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
}

fn main() {
    unsafe { env::set_var("CARGO_TERM_VERBOSE", "true") };

    if cfg!(target_os = "windows") {
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
