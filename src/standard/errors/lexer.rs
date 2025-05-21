use std::path::PathBuf;

pub enum ThrushLexerPanic {
    TooBigFile(PathBuf),
    TooMuchTokens,
}

impl ThrushLexerPanic {
    pub fn display(&self) -> String {
        match self {
            ThrushLexerPanic::TooBigFile(file_path) => {
                format!("\"{}\" is too big file.", file_path.display())
            }
            ThrushLexerPanic::TooMuchTokens => {
                String::from("The limit of 1_000_000 tokens has been exceeded.")
            }
        }
    }
}
