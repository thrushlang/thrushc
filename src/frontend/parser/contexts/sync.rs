#[derive(Debug, Clone, Copy, Default)]
pub enum ParserSyncPosition {
    Statement,
    Declaration,
    Expression,

    #[default]
    NoRelevant,
}
