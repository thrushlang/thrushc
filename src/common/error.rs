#[derive(Debug, Clone)]
pub enum ThrushCompilerError {
    Error(String, String, usize, Option<(usize, usize)>),
}
