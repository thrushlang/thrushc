use {
    super::super::error::ThrushCompilerError,
    super::objects::{Function, Local},
};

pub trait TokenLexemeBasics {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn parse_scapes(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Vec<u8>, ThrushCompilerError>;
}

pub trait FoundObjectEither {
    fn expected_local(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Local, ThrushCompilerError>;
    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushCompilerError>;
}
