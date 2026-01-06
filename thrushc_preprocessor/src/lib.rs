use thrushc_token::Token;

mod module;
mod parser;
mod signatures;
mod table;

#[derive(Debug)]
pub struct Preprocessor<'preprocessor> {
    tokens: &'preprocessor [Token],
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn new(tokens: &'preprocessor [Token]) -> Self {
        Self { tokens }
    }
}
