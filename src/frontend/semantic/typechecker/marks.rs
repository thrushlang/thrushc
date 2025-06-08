#![allow(clippy::upper_case_acronyms)]

#[derive(Debug, Clone, Copy, Default)]
pub enum TypeCheckerTypeCheckSource {
    Local,
    Call,
    LLI,

    #[default]
    NoRelevant,
}

impl TypeCheckerTypeCheckSource {
    pub fn is_local(&self) -> bool {
        matches!(self, TypeCheckerTypeCheckSource::Local)
    }

    pub fn is_lli(&self) -> bool {
        matches!(self, TypeCheckerTypeCheckSource::LLI)
    }
}
