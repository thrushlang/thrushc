use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::standard::logging::{self, LoggingType};

pub fn extract_code_from_file(file_path: &Path) -> Vec<u8> {
    match read_file_to_string_buffered(file_path) {
        Ok(code) => code,
        _ => {
            logging::log(
                LoggingType::Panic,
                &format!("'{}' file can't be read.", file_path.display()),
            );

            unreachable!()
        }
    }
}

fn read_file_to_string_buffered(path: &Path) -> Result<Vec<u8>, io::Error> {
    let file: File = File::open(path)?;
    let mut reader: BufReader<File> = BufReader::new(file);

    let mut buffer: Vec<u8> = Vec::with_capacity(100_000);
    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}
