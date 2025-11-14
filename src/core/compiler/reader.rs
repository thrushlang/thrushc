use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

pub fn get_file_source_code(file_path: &Path) -> String {
    match self::read_file_to_string_buffered(file_path) {
        Ok(code) => code,
        _ => {
            logging::print_any_panic(
                LoggingType::Panic,
                &format!("File '{}' can't be read correctly.", file_path.display()),
            );
        }
    }
}

fn read_file_to_string_buffered(path: &Path) -> Result<String, ()> {
    let file: File = File::open(path).map_err(|_| ())?;

    let mut reader: BufReader<File> = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::with_capacity(1_000_000_000);

    reader.read_to_end(&mut buffer).map_err(|_| ())?;

    Ok(String::from_utf8_lossy(&buffer).to_string())
}
