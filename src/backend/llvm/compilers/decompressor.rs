use std::{
    fs::{self, File, Permissions, write},
    io::BufReader,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use tar::Archive;
use xz2::bufread::XzDecoder;

use crate::standard::logging;

pub fn dump_x86_64_linux_clang(
    build_dir: &Path,
    raw_bytes: &[u8],
    tar_path: PathBuf,
    output_path: PathBuf,
) -> Result<PathBuf, ()> {
    let compilers_base_path: PathBuf = build_dir.join("compilers");
    let compilers_linux_base_path: PathBuf = build_dir.join("compilers").join("linux");

    let tar_path: PathBuf = compilers_linux_base_path.join(tar_path);
    let output_path: PathBuf = compilers_linux_base_path.join(output_path);

    if tar_path.exists() && output_path.exists() {
        return Ok(output_path);
    }

    let _ = fs::create_dir_all(&compilers_base_path);
    let _ = fs::create_dir_all(&compilers_linux_base_path);

    let _ = write(&tar_path, raw_bytes);

    if let Ok(file) = File::open(&tar_path) {
        let buff_reader: BufReader<File> = BufReader::new(file);
        let xz_decoded: XzDecoder<BufReader<File>> = XzDecoder::new(buff_reader);
        let mut tar_file: Archive<XzDecoder<BufReader<File>>> = Archive::new(xz_decoded);

        if tar_file.unpack(compilers_linux_base_path).is_ok() {
            if self::make_linux_executable(&output_path).is_ok() {
                return Ok(output_path);
            }

            logging::log(
                logging::LoggingType::Error,
                "Failed to make enbedded Clang executable.",
            );

            return Err(());
        }
    }

    logging::log(
        logging::LoggingType::Error,
        "Failed to extract enbedded Clang.",
    );

    Err(())
}

fn make_linux_executable(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut perms: Permissions = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}
