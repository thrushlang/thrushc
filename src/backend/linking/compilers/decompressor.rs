use std::{
    env,
    fs::{self, File, Permissions, write},
    io::BufReader,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use tar::Archive;
use xz2::bufread::XzDecoder;

use crate::standard::logging::{self, LoggingType};

pub fn dump_x86_64_linux_clang(
    clang_raw_manifest: &str,
    clang_raw_bytes: &[u8],
    clang_manifest_path: PathBuf,
    tar_path: PathBuf,
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
            let llvm_linux_backend: PathBuf = home_path.join("thrushlang/backends/llvm/linux");

            if !llvm_linux_backend.exists() {
                let _ = fs::create_dir_all(&llvm_linux_backend);
            }

            let tar_path: PathBuf = llvm_linux_backend.join(tar_path);
            let output_path: PathBuf = llvm_linux_backend.join(output_path);
            let manifest_path: PathBuf = llvm_linux_backend.join(clang_manifest_path);

            if !manifest_path.exists() {
                let _ = write(manifest_path, clang_raw_manifest);
            }

            if output_path.exists() {
                return Ok(output_path);
            }

            let _ = write(&tar_path, clang_raw_bytes);

            if let Ok(file) = File::open(&tar_path) {
                let buff_reader: BufReader<File> = BufReader::new(file);
                let xz_decoded: XzDecoder<BufReader<File>> = XzDecoder::new(buff_reader);
                let mut tar_file: Archive<XzDecoder<BufReader<File>>> = Archive::new(xz_decoded);

                if tar_file.unpack(llvm_linux_backend).is_ok() {
                    if self::make_linux_executable(&output_path).is_ok() {
                        return Ok(output_path);
                    }

                    logging::log(
                        logging::LoggingType::Error,
                        "Failed to make embedded Clang executable.",
                    );

                    return Err(());
                }
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
