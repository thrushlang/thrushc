#[derive(Debug)]
pub struct AnalyzerContext {
    loop_depth: u32,
    global_assembler: bool,
}

impl AnalyzerContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            loop_depth: 0,
            global_assembler: false,
        }
    }
}

impl AnalyzerContext {
    #[inline]
    pub fn increment_loop_depth(&mut self) {
        self.loop_depth += 1;
    }

    #[inline]
    pub fn decrement_loop_depth(&mut self) {
        self.loop_depth -= 1;
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
    pub fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }

    #[inline]
    pub fn has_global_assembler(&self) -> bool {
        self.global_assembler
    }
}
