use crate::core::compiler::backends::llvm::flavors::LLVMLinkerFlavor;

#[derive(Debug, Default)]
pub enum LinkerModeType {
    #[default]
    None,

    LLVMLinker,
}

#[derive(Debug, Default)]
pub enum LinkerConfiguration {
    #[default]
    None,

    LLVMLinker(LLVMLinkerFlavor),
}

#[derive(Debug)]
pub struct LinkerMode {
    args: Vec<String>,
    status: bool,
    kind: LinkerModeType,
    config: LinkerConfiguration,
}

impl LinkerMode {
    #[inline]
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            status: false,

            kind: LinkerModeType::default(),
            config: LinkerConfiguration::default(),
        }
    }
}

impl LinkerMode {
    #[inline]
    pub fn add_arg(&mut self, arg: String) {
        self.args.push(arg);
    }
}

impl LinkerMode {
    #[inline]
    pub fn get_args(&self) -> &[String] {
        &self.args
    }

    #[inline]
    pub fn get_status(&self) -> bool {
        self.status
    }

    #[inline]
    pub fn get_linker_type(&self) -> &LinkerModeType {
        &self.kind
    }

    #[inline]
    pub fn get_config(&self) -> &LinkerConfiguration {
        &self.config
    }
}

impl LinkerMode {
    #[inline]
    pub fn turn_on(&mut self, linker_type: LinkerModeType) {
        self.status = true;
        self.kind = linker_type;
    }

    #[inline]
    pub fn set_up_config(&mut self, config: LinkerConfiguration) {
        self.config = config;
    }
}

impl LinkerModeType {
    #[inline]
    pub fn is_llvm_linker(&self) -> bool {
        matches!(self, LinkerModeType::LLVMLinker)
    }
}
