use {
    super::super::error::ThrushError,
    super::objects::{Function, Local},
};

pub trait TokenLexeme {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
}

pub trait FoundObjectEither {
    fn expected_local(&self, line: usize, span: (usize, usize)) -> Result<&Local, ThrushError>;
    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushError>;
}
