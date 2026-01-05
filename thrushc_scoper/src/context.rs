#[derive(Debug, Clone, Copy)]
pub struct ScoperContext {
    loop_depth: u32,
    inside_function: bool,
}

impl ScoperContext {
    #[inline]
    pub fn new() -> Self {
        ScoperContext {
            loop_depth: 0,
            inside_function: false,
        }
    }
}

impl ScoperContext {
    #[inline]
    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    #[inline]
    pub fn leave_loop(&mut self) {
        self.loop_depth -= 1;
    }

    #[inline]
    pub fn enter_function(&mut self) {
        self.inside_function = true;
    }

    #[inline]
    pub fn leave_function(&mut self) {
        self.inside_function = false;
    }
}

impl ScoperContext {
    #[inline]
    pub fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }

    #[inline]
    pub fn is_inside_function(&self) -> bool {
        self.inside_function
    }
}
