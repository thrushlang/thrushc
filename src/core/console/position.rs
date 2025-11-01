#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,

    InternalLinker,
    External,
}

impl CommandLinePosition {
    #[inline]
    pub fn at_external(&self) -> bool {
        matches!(self, CommandLinePosition::External)
    }

    #[inline]
    pub fn at_internal_linker(&self) -> bool {
        matches!(self, CommandLinePosition::InternalLinker)
    }
}
