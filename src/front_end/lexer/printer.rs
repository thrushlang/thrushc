use std::{
    fmt::{self, Display, Write as WriteIO},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    core::console::logging,
    front_end::{lexer::token::Token, types::lexer::types::Tokens, utils::rand},
};

pub fn print_to_file(tokens: &Tokens, build_path: &Path, file_name: &str) -> Result<(), io::Error> {
    let base_tokens_path: PathBuf = build_path.join("emit").join("tokens");

    if !base_tokens_path.exists() {
        let _ = std::fs::create_dir_all(&base_tokens_path);
    }

    let formatted_file_name: String =
        format!("{}_{}.tokens", rand::generate_random_string(), file_name);

    let file_path: PathBuf = base_tokens_path.join(formatted_file_name);

    let mut file: File = File::create(&file_path)?;

    tokens
        .iter()
        .try_for_each(|token| writeln!(file, "{}", token))?;

    Ok(())
}

pub fn generate_string(tokens: &Tokens) -> Result<String, fmt::Error> {
    let mut buffer: String = String::with_capacity(tokens.len());

    tokens
        .iter()
        .try_for_each(|token| writeln!(buffer, "{}", token))?;

    Ok(buffer)
}

pub fn print_to_stdout_fine(tokens: &Tokens, file_name: &str) -> Result<(), fmt::Error> {
    let mut tokens_formatted: String = String::with_capacity(tokens.len());

    tokens
        .iter()
        .try_for_each(|token| writeln!(tokens_formatted, "{}", token))?;

    logging::write(
        logging::OutputIn::Stdout,
        &format!("\n{}\n\n", file_name.bright_green().bold()),
    );

    logging::write(logging::OutputIn::Stdout, &tokens_formatted);
    logging::write(logging::OutputIn::Stdout, "\n");

    Ok(())
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOKEN TYPE '{:?}' literal '{}', ascii '{}' at '{}'.",
            self.kind, self.lexeme, self.ascii, self.span
        )
    }
}
