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
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                &format!("File '{}' can't be read correctly.", file_path.display()),
            );
        }
    }
}

fn read_file_to_string_buffered(path: &Path) -> Result<String, ()> {
    if let Ok(file) = File::open(path) {
        let mut reader: BufReader<File> = BufReader::new(file);

        let mut buffer: Vec<u8> = Vec::with_capacity(1_000_000);

        if reader.read_to_end(&mut buffer).is_err() {
            return Err(());
        }

        if let Ok(code) = String::from_utf8(buffer) {
            return Ok(code);
        }
    }

    Err(())
}
