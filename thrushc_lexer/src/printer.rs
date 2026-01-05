use std::fmt::Write as WriteIO;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;
use thrushc_logging::OutputIn;
use thrushc_token::Token;

pub fn print_to_file(
    tokens: &[Token],
    build_path: &Path,
    file_name: &str,
) -> Result<(), io::Error> {
    let base_tokens_path: PathBuf = build_path.join("emit").join("tokens");

    if !base_tokens_path.exists() {
        let _ = std::fs::create_dir_all(&base_tokens_path);
    }

    let formatted_file_name: String =
        format!("{}_{}.tokens", self::generate_random_string(), file_name);

    let file_path: PathBuf = base_tokens_path.join(formatted_file_name);

    let mut file: File = File::create(&file_path)?;

    tokens
        .iter()
        .try_for_each(|token| writeln!(file, "{}", token))?;

    Ok(())
}

pub fn generate_string(tokens: &[Token]) -> Result<String, std::fmt::Error> {
    let mut buffer: String = String::with_capacity(tokens.len());

    tokens
        .iter()
        .try_for_each(|token| writeln!(buffer, "{}", token))?;

    Ok(buffer)
}

pub fn print_to_stdout_fine(tokens: &[Token], file_name: &str) -> Result<(), std::fmt::Error> {
    let mut tokens_formatted: String = String::with_capacity(tokens.len());

    tokens
        .iter()
        .try_for_each(|token| writeln!(tokens_formatted, "{}", token))?;

    thrushc_logging::write(
        OutputIn::Stdout,
        &format!("\n{}\n\n", file_name.bright_green().bold()),
    );

    thrushc_logging::write(OutputIn::Stdout, &tokens_formatted);
    thrushc_logging::write(OutputIn::Stdout, "\n");

    Ok(())
}

fn generate_random_string() -> String {
    let length: usize = fastrand::usize(5..=10);

    let mut random_string: String = String::with_capacity(length);

    for _ in 0..length {
        match fastrand::u8(0..62) {
            n @ 0..=9 => random_string.push((b'0' + n) as char),
            n @ 10..=35 => random_string.push((b'A' + n - 10) as char),
            n @ 36..=61 => random_string.push((b'a' + n - 36) as char),

            _ => random_string.push(b'_' as char),
        }
    }

    random_string
}
