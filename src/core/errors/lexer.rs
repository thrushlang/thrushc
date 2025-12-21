#[derive(Debug)]
pub enum LexerPanic {
    TooBigFile(std::path::PathBuf),
    TooMuchTokens,
}

impl LexerPanic {
    #[inline]
    pub fn display(&self) -> String {
        match self {
            LexerPanic::TooBigFile(file_path) => {
                format!("\"{}\" is too big file.", file_path.display())
            }
            LexerPanic::TooMuchTokens => {
                String::from("The limit of 1_000_000_000 tokens has been exceeded.")
            }
        }
    }
}
