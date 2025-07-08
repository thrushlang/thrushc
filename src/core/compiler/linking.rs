use std::path::PathBuf;

#[derive(Debug)]
pub struct LinkingCompilersConfiguration {
    use_clang: bool,
    use_gcc: bool,
    compiler_args: Vec<String>,
    custom_gcc: Option<PathBuf>,
    custom_clang: Option<PathBuf>,
    debug_clang_commands: bool,
    debug_gcc_commands: bool,
}

impl LinkingCompilersConfiguration {
    pub fn new() -> Self {
        Self {
            use_clang: false,
            use_gcc: false,
            compiler_args: Vec::with_capacity(50),
            custom_gcc: None,
            custom_clang: None,
            debug_clang_commands: false,
            debug_gcc_commands: false,
        }
    }

    pub fn get_args(&self) -> &[String] {
        &self.compiler_args
    }

    pub fn get_custom_clang(&self) -> Option<&PathBuf> {
        self.custom_clang.as_ref()
    }

    pub fn get_debug_clang_commands(&self) -> bool {
        self.debug_clang_commands
    }

    pub fn get_debug_gcc_commands(&self) -> bool {
        self.debug_gcc_commands
    }

    pub fn get_custom_gcc(&self) -> Option<&PathBuf> {
        self.custom_gcc.as_ref()
    }

    pub fn get_use_clang(&self) -> bool {
        self.use_clang
    }

    pub fn get_use_gcc(&self) -> bool {
        self.use_gcc
    }
}

impl LinkingCompilersConfiguration {
    pub fn set_use_clang(&mut self, value: bool) {
        self.use_clang = value;
    }

    pub fn set_use_gcc(&mut self, value: bool) {
        self.use_gcc = value;
    }

    pub fn set_custom_clang(&mut self, value: PathBuf) {
        self.custom_clang = Some(value);
    }

    pub fn set_custom_gcc(&mut self, value: PathBuf) {
        self.custom_gcc = Some(value);
    }

    pub fn set_debug_clang_commands(&mut self, value: bool) {
        self.debug_clang_commands = value;
    }

    pub fn set_debug_gcc_commands(&mut self, value: bool) {
        self.debug_gcc_commands = value;
    }
}

impl LinkingCompilersConfiguration {
    pub fn add_compiler_arg(&mut self, value: String) {
        self.compiler_args.push(value);
    }
}
