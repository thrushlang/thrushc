use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::frontends::classical::{lexer::token::Token, types::lexer::types::Tokens, utils::rand};

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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOKEN TYPE '{:?}' literal '{}', ascii '{}' at '{}'.",
            self.kind, self.lexeme, self.ascii_lexeme, self.span
        )
    }
}
