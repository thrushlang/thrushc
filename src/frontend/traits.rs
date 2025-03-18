use {
    super::super::error::ThrushError,
    super::objects::{Function, Local},
};

pub trait TokenLexeme {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn parse_scapes(&self, line: usize, span: (usize, usize)) -> Result<Vec<u8>, ThrushError>;
}

pub trait FoundObjectEither {
    fn expected_local(&self, line: usize, span: (usize, usize)) -> Result<&Local, ThrushError>;
    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushError>;
}
