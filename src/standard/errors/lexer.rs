use std::path::PathBuf;

pub enum ThrushLexerPanic {
    TooBigFile(PathBuf),
}

impl ThrushLexerPanic {
    pub fn display(&self) -> String {
        match self {
            ThrushLexerPanic::TooBigFile(file_path) => {
                format!("\"{}\" is too big", file_path.display())
            }
        }
    }
}
