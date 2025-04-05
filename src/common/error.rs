#[derive(Debug)]
pub enum ThrushCompilerError {
    Error(String, String, usize, Option<(usize, usize)>),
}
