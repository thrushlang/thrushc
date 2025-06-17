#[cfg(target_os = "linux")]
use {fs::Permissions, std::os::unix::fs::PermissionsExt};

use std::{
    env,
    fs::{self, File, write},
    io::BufReader,
    path::{Path, PathBuf},
};

use tar::Archive;
use xz2::bufread::XzDecoder;

#[cfg(target_os = "windows")]
use std::io::{self};

#[cfg(target_os = "windows")]
use zip::{ZipArchive, read::ZipFile};

use crate::core::console::logging::{self, LoggingType};

#[cfg(target_os = "linux")]
pub fn dump_x86_64_clang_linux(
    clang_raw_manifest: &str,
    clang_raw_bytes: &[u8],
    clang_manifest_path: PathBuf,
    compressed_file_path: PathBuf,
    output_path: PathBuf,
) -> Result<PathBuf, ()> {
    let raw_home_path: String = env::var("HOME").unwrap_or_else(|_| {
        logging::log(LoggingType::Panic, "Unable to get %HOME% path at linux.");
        unreachable!()
    });

    let home_path: PathBuf = PathBuf::from(raw_home_path);

    if home_path.exists() {
        let llvm_backend: PathBuf = home_path.join("thrushlang/backends/llvm/linux");

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
            let buff_reader: BufReader<File> = BufReader::new(file);
            let xz_decoded: XzDecoder<BufReader<File>> = XzDecoder::new(buff_reader);
            let mut tar_file: Archive<XzDecoder<BufReader<File>>> = Archive::new(xz_decoded);

            if tar_file.unpack(llvm_backend).is_ok() {
                if self::make_linux_executable(&output_path).is_ok() {
                    return Ok(output_path);
                }

                logging::log(
                    logging::LoggingType::Error,
                    "Failed to make Clang executable at linux.",
                );

                return Err(());
            }

            logging::log(
                logging::LoggingType::Error,
                "Failed to decompress Clang executable at linux.",
            );

            return Err(());
        }

        logging::log(
            logging::LoggingType::Error,
            "Failed to get Clang compressed at linux.",
        );

        return Err(());
    }

    logging::log(
        logging::LoggingType::Error,
        "%HOME% path not exist at linux.",
    );

    Err(())
}

#[cfg(target_os = "windows")]
pub fn dump_x86_64_clang_windows(
    clang_raw_manifest: &str,
    clang_raw_bytes: &[u8],
    clang_manifest_path: PathBuf,
    compressed_file_path: PathBuf,
    output_path: PathBuf,
) -> Result<PathBuf, ()> {
    let raw_home_path: String = env::var("APPDATA").unwrap_or_else(|_| {
        logging::log(
            LoggingType::Panic,
            "Unable to get %APPDATA% path at windows.",
        );
        unreachable!()
    });

    let home_path: PathBuf = PathBuf::from(raw_home_path);

    if home_path.exists() {
        let llvm_backend: PathBuf = home_path.join("thrushlang/backends/llvm/windows");

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

        if File::open(&compressed_file).is_err() {
            if self::decompress_zip(compressed_file, llvm_backend).is_ok() {
                let canonical_windows_clang_path: PathBuf = output_path.join("bin/clang.exe");

                if canonical_windows_clang_path.exists() {
                    return Ok(canonical_windows_clang_path);
                }

                logging::log(
                    logging::LoggingType::Error,
                    "Failed to get Clang executable at windows.",
                );

                return Err(());
            }

            logging::log(
                logging::LoggingType::Error,
                "Failed to decompress Clang at windows.",
            );

            return Err(());
        }

        logging::log(
            logging::LoggingType::Error,
            "Failed to open Clang compressed at windows.",
        );

        return Err(());
    }

    logging::log(
        logging::LoggingType::Error,
        "%APPDATA% path not exist at windows.",
    );

    Err(())
}

#[cfg(target_os = "linux")]
fn make_linux_executable(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut perms: Permissions = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn decompress_zip(zip_path: PathBuf, extract_to: PathBuf) -> zip::result::ZipResult<()> {
    let file: File = File::open(zip_path)?;
    let mut archive: ZipArchive<File> = ZipArchive::new(file)?;

    fs::create_dir_all(&extract_to)?;

    for i in 0..archive.len() {
        let mut file: ZipFile = archive.by_index(i)?;
        let outpath: PathBuf = Path::new(&extract_to).join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
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
