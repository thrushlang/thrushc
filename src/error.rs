#[derive(Debug)]
pub enum ThrushError {
    Error(String, String, usize, String),
}
