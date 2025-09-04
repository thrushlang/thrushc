#[derive(Debug, Copy, Clone)]
pub enum TypeCheckerPosition {
    Local,
}

impl TypeCheckerPosition {
    pub fn at_local(&self) -> bool {
        matches!(self, TypeCheckerPosition::Local)
    }
}
