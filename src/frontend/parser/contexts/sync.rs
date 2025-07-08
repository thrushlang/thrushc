#[derive(Debug, Clone, Copy)]
pub enum SyncPosition {
    Statement,
    Declaration,
    Expression,
    NoRelevant,
}
