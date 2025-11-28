use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use isahc::Body;
use isahc::HttpClient;
use isahc::ReadResponseExt;
use isahc::Response;
use isahc::config::Configurable;
use isahc::config::RedirectPolicy;

use serde_json::Value;

use crate::constants;
use crate::logging;
use crate::logging::LoggingType;
use crate::options::BuildOptions;

#[derive(Debug)]
pub struct CompilerBuilderDependencies<'a> {
    options: &'a BuildOptions,
}

impl<'a> CompilerBuilderDependencies<'a> {
    #[inline]
    pub fn new(options: &'a BuildOptions) -> Self {
        Self { options }
    }
}

impl<'a> CompilerBuilderDependencies<'a> {
    pub fn install(&self) {
        self.reset_llvm_build_path();

        if let Err(err) = self.install_llvm_c_api() {
            logging::log(LoggingType::Panic, &err);
        }

        logging::log(LoggingType::Log, "LLVM installed.\n\n");
    }
}

impl CompilerBuilderDependencies<'_> {
    fn install_llvm_c_api(&self) -> Result<(), String> {
        logging::log(LoggingType::Log, "Installing LLVM...\n");

        let mut llvm_c_apis: Response<Body> = isahc::get(constants::COMPILER_TOOLCHAINS)
            .map_err(|_| "Could not make request to GitHub releases API.".to_string())?;

        let llvm_c_apis_json: Value = llvm_c_apis
            .json::<Value>()
            .map_err(|_| "Could not parse response as JSON.".to_string())?;

        let all_llvm_c_apis: &Vec<Value> = llvm_c_apis_json
            .as_array()
            .ok_or("Expected array in JSON response.")?;

        let tag_name_to_find: &str = &self.get_options().get_llvm_host_triple();
        let tag_name_pattern_to_replace: &str =
            &self.get_options().get_llvm_host_triple_versioned();

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
                            let version_string: String =
                                tag_name.replace(tag_name_pattern_to_replace, "");

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

        latest_llvm_c_api
            .and_then(|obj| obj.as_object())
            .and_then(|obj| obj.get("tag_name"))
            .and_then(|v| v.as_str())
            .map(|tag| {
                logging::log(
                    LoggingType::Log,
                    &format!("Installing '{}'.\n", tag_name_to_find),
                );

                Some(tag)
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

        Err("No compatible LLVM releases found.".to_string())
    }
}

impl CompilerBuilderDependencies<'_> {
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
            let full_path: PathBuf = self.get_options().get_llvm_build_path().join(name);

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

            let bytes: Vec<u8> = response
                .bytes()
                .map_err(|e| format!("Failed to read response for {}: {}", link, e))?;

            let mut file: std::fs::File = std::fs::File::create(&full_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", full_path, e))?;

            std::io::Write::write_all(&mut file, &bytes)
                .map_err(|e| format!("Failed to write to file {:?}: {}", full_path, e))?;
        }

        for name in names.iter() {
            let full_path: PathBuf = self.get_options().get_llvm_build_path().join(name);

            if full_path
                .extension()
                .is_some_and(|extension| extension == "xz" || extension == "zip")
            {
                self.llvm_decompress_file(&full_path)?;
                let _ = std::fs::remove_file(&full_path);
            }
        }

        if std::env::consts::FAMILY == "unix" {
            self::make_unix_executable(
                &self
                    .get_options()
                    .get_llvm_build_path()
                    .join("bin/llvm-config"),
            )?;
        }

        Ok(())
    }
}

impl CompilerBuilderDependencies<'_> {
    fn llvm_decompress_file(&self, file_path: &Path) -> Result<(), String> {
        let mut tar_command: Command = Command::new("tar");

        tar_command
            .arg("-xf")
            .arg("--strip-components=1")
            .arg(file_path)
            .arg("-C")
            .arg(self.get_options().get_llvm_build_path());

        if let Ok(tar_output) = tar_command.output() {
            let stderr: Cow<'_, str> = String::from_utf8_lossy(&tar_output.stderr);

            if !tar_output.status.success() {
                logging::log(LoggingType::Error, &stderr);
            }

            return Ok(());
        }

        Err(format!(
            "'{}' could not be decompressed.",
            file_path.display()
        ))
    }
}

impl CompilerBuilderDependencies<'_> {
    #[inline]
    fn reset_llvm_build_path(&self) {
        let _ = std::fs::remove_dir_all(self.get_options().get_llvm_build_path());
        let _ = std::fs::create_dir_all(self.get_options().get_llvm_build_path());
    }
}

impl CompilerBuilderDependencies<'_> {
    #[inline]
    pub fn get_options(&self) -> &BuildOptions {
        self.options
    }
}

fn make_unix_executable(file_path: &Path) -> Result<(), String> {
    let mut chmod_command: Command = Command::new("chmod");

    chmod_command.arg("+x").arg(file_path);

    if let Ok(chmod_output) = chmod_command.output() {
        let stderr: Cow<'_, str> = String::from_utf8_lossy(&chmod_output.stderr);

        if !chmod_output.status.success() {
            logging::log(LoggingType::Error, &stderr);
        }

        return Ok(());
    }

    Err(format!(
        "'{}' could not be made executable.",
        file_path.display()
    ))
}
