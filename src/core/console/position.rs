#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,

    External,
}

impl CommandLinePosition {
    #[inline]
    pub fn at_external(&self) -> bool {
        matches!(self, CommandLinePosition::External)
    }
}
