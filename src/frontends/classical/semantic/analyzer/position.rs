#[derive(Debug, Copy, Clone)]
pub enum TypeCheckerPosition {
    Local,
}

impl TypeCheckerPosition {
    #[inline]
    pub fn at_local(&self) -> bool {
        matches!(self, TypeCheckerPosition::Local)
    }
}
