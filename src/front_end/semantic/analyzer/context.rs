#[derive(Debug)]
pub struct AnalyzerContext {
    global_assembler: bool,
}

impl AnalyzerContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            global_assembler: false,
        }
    }
}

impl AnalyzerContext {
    #[inline]
    pub fn set_has_global_assembler(&mut self) {
        self.global_assembler = true;
    }
}

impl AnalyzerContext {
    #[inline]
    pub fn has_global_assembler(&self) -> bool {
        self.global_assembler
    }
}
