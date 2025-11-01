#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,

    InternalLinker,
    ExternalCompiler,
}

impl CommandLinePosition {
    #[inline]
    pub fn at_external_linking_compiler(&self) -> bool {
        matches!(self, CommandLinePosition::ExternalCompiler)
    }

    #[inline]
    pub fn at_internal_linker(&self) -> bool {
        matches!(self, CommandLinePosition::InternalLinker)
    }
}
