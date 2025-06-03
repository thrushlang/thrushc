use std::{
    env,
    fs::{self, File, Permissions, write},
    io::{self, BufReader},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use tar::Archive;
use xz2::bufread::XzDecoder;
use zip::{ZipArchive, read::ZipFile};

use crate::standard::logging::{self, LoggingType};

pub fn dump_x86_64_clang(
    clang_raw_manifest: &str,
    clang_raw_bytes: &[u8],
    clang_manifest_path: PathBuf,
    compressed_file_path: PathBuf,
    output_path: PathBuf,
) -> Result<PathBuf, ()> {
    let home_path: Result<String, env::VarError> = if cfg!(target_os = "linux") {
        env::var("HOME")
    } else if cfg!(target_os = "windows") {
        env::var("APPDATA")
    } else {
        logging::log(LoggingType::Panic, "Unable to get system user HOME path.");
        unreachable!()
    };

    if let Ok(raw_home_path) = home_path {
        let home_path: PathBuf = PathBuf::from(raw_home_path);

        if home_path.exists() {
            let llvm_backend: PathBuf = if cfg!(target_os = "linux") {
                home_path.join("thrushlang/backends/llvm/linux")
            } else {
                home_path.join("thrushlang/backends/llvm/windows")
            };

            if !llvm_backend.exists() {
                let _ = fs::create_dir_all(&llvm_backend);
            }

            let compressed_file: PathBuf = llvm_backend.join(compressed_file_path);
            let output_path: PathBuf = llvm_backend.join(output_path);
            let manifest_path: PathBuf = llvm_backend.join(clang_manifest_path);

            if !manifest_path.exists() {
                let _ = write(manifest_path, clang_raw_manifest);
            }

            if output_path.exists() {
                return Ok(output_path);
            }

            let _ = write(&compressed_file, clang_raw_bytes);

            if let Ok(file) = File::open(&compressed_file) {
                if cfg!(target_os = "linux") {
                    let buff_reader: BufReader<File> = BufReader::new(file);
                    let xz_decoded: XzDecoder<BufReader<File>> = XzDecoder::new(buff_reader);
                    let mut tar_file: Archive<XzDecoder<BufReader<File>>> =
                        Archive::new(xz_decoded);

                    if tar_file.unpack(llvm_backend).is_ok() {
                        if self::make_linux_executable(&output_path).is_ok() {
                            return Ok(output_path);
                        }

                        logging::log(
                            logging::LoggingType::Error,
                            "Failed to make Clang executable at Linux.",
                        );

                        return Err(());
                    }

                    logging::log(
                        logging::LoggingType::Error,
                        "Failed to decompress Clang executable at Linux.",
                    );
                } else if cfg!(target_os = "windows") {
                    if self::decompress_zip(compressed_file, llvm_backend).is_ok() {
                        let canonical_windows_clang_path: PathBuf = output_path.join("clang.exe");

                        if canonical_windows_clang_path.exists() {
                            return Ok(canonical_windows_clang_path);
                        }

                        logging::log(
                            logging::LoggingType::Error,
                            "Failed to get Clang executable at Windows.",
                        );

                        return Err(());
                    }

                    logging::log(
                        logging::LoggingType::Error,
                        "Failed to decompress Clang at Windows.",
                    );
                }

                return Err(());
            }
        }

        logging::log(logging::LoggingType::Error, "System HOME path not exist.");
    }

    logging::log(
        logging::LoggingType::Error,
        "Failed to extract embedded Clang x86_64 Linux.",
    );

    Err(())
}

fn make_linux_executable(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut perms: Permissions = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

fn decompress_zip(zip_path: PathBuf, extract_to: PathBuf) -> zip::result::ZipResult<()> {
    let file: File = File::open(zip_path)?;
    let mut archive: ZipArchive<File> = ZipArchive::new(file)?;

    fs::create_dir_all(&extract_to)?;

    for i in 0..archive.len() {
        let mut file: ZipFile = archive.by_index(i)?;
        let outpath: PathBuf = Path::new(&extract_to).join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile: File = File::create(&outpath)?;

            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
