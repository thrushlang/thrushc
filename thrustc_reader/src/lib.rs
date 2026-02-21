use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use encoding_rs::CoderResult;
use encoding_rs::Encoding;

use thrustc_logging::LoggingType;

pub fn get_file_source_code(file_path: &std::path::Path) -> String {
    if let Ok(total_lines) = self::count_lines_exact(file_path) {
        if total_lines > 100_000 {
            thrustc_logging::print_warn(
                LoggingType::Warning,
                &format!(
                    "File '{}' exceeds 100000 lines of code. You should split it into as a minimal two diferent files.",
                    file_path.display()
                ),
            );

            return Default::default();
        }
    }

    self::read_file_to_string_buffered(file_path).unwrap_or_else(|error| {
        thrustc_logging::print_critical_error(
            LoggingType::Error,
            &format!(
                "File '{}' can't be read correctly, because: '{}'.",
                file_path.display(),
                error
            ),
        )
    })
}

fn read_file_to_string_buffered(path: &std::path::Path) -> Result<String, &str> {
    let file: std::fs::File = std::fs::File::open(path).map_err(|_| "unable to open the file")?;

    let mut reader: BufReader<std::fs::File> = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::with_capacity(u8::MAX as usize);

    reader
        .read_to_end(&mut buffer)
        .map_err(|_| "unable to fill the buffer")?;

    let (encoding, offset) = Encoding::for_bom(&buffer).unwrap_or((encoding_rs::UTF_8, 0));

    if encoding == encoding_rs::UTF_8 {
        String::from_utf8(buffer).map_err(|_| "invalid utf-8 content")
    } else {
        let bytes: &[u8] = &buffer[offset..];
        let mut decoder: encoding_rs::Decoder = encoding.new_decoder();
        let mut string: String = String::with_capacity(bytes.len() * 2);

        let (result, _, _) = decoder.decode_to_string(bytes, &mut string, true);

        if let CoderResult::InputEmpty = result {
            Ok(string)
        } else {
            Err("Unable to decode correctly to utf-8")
        }
    }
}

fn count_lines_exact(path: &Path) -> anyhow::Result<u64> {
    let mut file: std::fs::File = std::fs::File::open(path)?;
    count_lines_from_reader(&mut file)
}

fn count_lines_from_reader<R: Read>(reader: &mut R) -> anyhow::Result<u64> {
    const CHUNK_SIZE: usize = 1 << 16;

    let mut buffer: [u8; _] = [0u8; CHUNK_SIZE];
    let mut count: u64 = 0;

    loop {
        let bytes_read: usize = reader.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        count += bytecount::count(&buffer[..bytes_read], b'\n') as u64;
    }

    Ok(count)
}
