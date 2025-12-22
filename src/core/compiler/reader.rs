use std::io::BufReader;
use std::io::Read;

use encoding_rs::CoderResult;
use encoding_rs::Encoding;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

pub fn get_file_source_code(file_path: &std::path::Path) -> String {
    match self::read_file_to_string_buffered(file_path) {
        Ok(code) => code,
        _ => {
            logging::print_critical_error(
                LoggingType::Error,
                &format!("File '{}' can't be read correctly.", file_path.display()),
            );
        }
    }
}

fn read_file_to_string_buffered(path: &std::path::Path) -> Result<String, ()> {
    let file: std::fs::File = std::fs::File::open(path).map_err(|_| ())?;

    let mut reader: BufReader<std::fs::File> = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::with_capacity(1_000_000_000);

    reader.read_to_end(&mut buffer).map_err(|_| ())?;

    let (encoding, offset) = Encoding::for_bom(&buffer).unwrap_or((encoding_rs::UTF_8, 0));

    if encoding == encoding_rs::UTF_8 {
        String::from_utf8(buffer).map_err(|_| ())
    } else {
        let bytes: &[u8] = &buffer[offset..];
        let mut decoder: encoding_rs::Decoder = encoding.new_decoder();
        let mut string: String = String::with_capacity(bytes.len() * 2);

        let (result, _, _) = decoder.decode_to_string(bytes, &mut string, true);

        if let CoderResult::InputEmpty = result {
            Ok(string)
        } else {
            Err(())
        }
    }
}
