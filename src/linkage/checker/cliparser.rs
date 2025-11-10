use crate::core::compiler::linking::LinkingCompilersConfiguration;

#[derive(Debug)]
pub struct CompilerLinkerParser<'compiler_linker_parser> {
    config: &'compiler_linker_parser LinkingCompilersConfiguration,

    libraries: Vec<String>,
    search_paths: Vec<String>,
}

impl<'compiler_linker_parser> CompilerLinkerParser<'compiler_linker_parser> {
    #[inline]
    pub fn new(config: &'compiler_linker_parser LinkingCompilersConfiguration) -> Self {
        Self {
            config,
            libraries: Vec::with_capacity(10),
            search_paths: Vec::with_capacity(10),
        }
    }
}

impl CompilerLinkerParser<'_> {
    pub fn parse(&mut self) -> (&[String], &[String]) {
        if self.config.get_use_clang() {
            self.parse_clang();
        }

        if self.config.get_use_gcc() {
            self.parse_gcc();
        }

        (self.search_paths.as_slice(), self.libraries.as_slice())
    }
}

impl CompilerLinkerParser<'_> {
    fn parse_clang(&mut self) {
        let mut current: usize = 0;

        let max_args: usize = self.config.get_args().len();

        while current < max_args {
            if let Some(flag) = self.config.get_args().get(current) {
                if flag.starts_with("-l") {
                    self.add_library(
                        CompilerLinkerParser::normalize_lib_name(flag.trim_start_matches("-l"))
                            .to_string(),
                    );
                } else if flag.starts_with("-L") {
                    self.add_library(flag.trim_start_matches("-L").to_string());
                }
            }

            current += 1;
        }
    }

    fn parse_gcc(&mut self) {
        let mut current: usize = 0;

        let max_args: usize = self.config.get_args().len();

        while current < max_args {
            if let Some(flag) = self.config.get_args().get(current) {
                if let Some(lib) = flag.strip_prefix("--library=") {
                    self.add_library(CompilerLinkerParser::normalize_lib_name(lib));
                } else if let Some(search_path) = flag.strip_prefix("--library-path=") {
                    self.add_search_path(search_path.to_string());
                }
            }

            current += 1;
        }
    }
}

impl CompilerLinkerParser<'_> {
    fn normalize_lib_name(name: &str) -> String {
        name.strip_prefix("lib")
            .unwrap_or(name)
            .strip_suffix(".so")
            .unwrap_or(name)
            .strip_suffix(".a")
            .unwrap_or(name)
            .to_string()
    }
}

impl CompilerLinkerParser<'_> {
    #[inline]
    pub fn add_library(&mut self, lib: String) {
        self.libraries.push(lib);
    }

    #[inline]
    pub fn add_search_path(&mut self, search_path: String) {
        self.search_paths.push(search_path);
    }
}
