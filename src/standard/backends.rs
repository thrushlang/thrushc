use std::path::PathBuf;

use inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple};

use crate::standard::logging;

use super::misc::{Emitable, ThrushOptimization};

/* ######################################################################


    LLVM BACKEND - START


########################################################################*/

#[derive(Debug, Clone, Copy)]
pub enum LLVMModificatorPasses {
    LoopVectorization,
    LoopUnroll,
    LoopInterleaving,
    LoopSimplifyVectorization,
    MergeFunctions,
}

#[derive(Debug)]
pub struct CompilersConfiguration {
    use_clang: bool,
    use_gcc: bool,
    compiler_args: Vec<String>,
    custom_gcc: Option<PathBuf>,
    custom_clang: Option<PathBuf>,
}

#[derive(Debug)]
pub struct LLVMBackend {
    target_cpu: String,
    target_triple: TargetTriple,
    optimization: ThrushOptimization,
    emit: Vec<Emitable>,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    modificator_passes: Vec<LLVMModificatorPasses>,
    opt_passes: String,
    compilers_config: CompilersConfiguration,
}

impl LLVMBackend {
    pub fn new() -> Self {
        Self {
            target_cpu: String::with_capacity(100),
            target_triple: TargetMachine::get_default_triple(),
            optimization: ThrushOptimization::None,
            emit: Vec::with_capacity(10),
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            modificator_passes: Vec::with_capacity(10),
            opt_passes: String::with_capacity(100),
            compilers_config: CompilersConfiguration::new(),
        }
    }

    pub fn get_reloc_mode(&self) -> RelocMode {
        self.reloc_mode
    }

    pub fn get_code_model(&self) -> CodeModel {
        self.code_model
    }

    pub fn set_optimization(&mut self, opt: ThrushOptimization) {
        self.optimization = opt;
    }

    pub fn set_reloc_mode(&mut self, reloc_mode: RelocMode) {
        self.reloc_mode = reloc_mode;
    }

    pub fn set_code_model(&mut self, code_model: CodeModel) {
        self.code_model = code_model;
    }

    pub fn set_target_triple(&mut self, target_triple: TargetTriple) {
        self.target_triple = target_triple;
    }

    pub fn add_emit_option(&mut self, emit: Emitable) {
        self.emit.push(emit);
    }

    pub fn get_target_triple(&self) -> &TargetTriple {
        &self.target_triple
    }

    pub fn get_optimization(&self) -> ThrushOptimization {
        self.optimization
    }

    pub fn was_emited(&self) -> bool {
        !self.emit.is_empty()
    }

    pub fn contains_emitable(&self, emit: Emitable) -> bool {
        self.emit.contains(&emit)
    }

    pub fn get_target_cpu(&self) -> &str {
        self.target_cpu.as_str()
    }

    pub fn set_target_cpu(&mut self, target_cpu: String) {
        self.target_cpu = target_cpu;
    }

    pub fn get_opt_passes(&self) -> &str {
        self.opt_passes.as_str()
    }

    pub fn set_opt_passes(&mut self, opt_passes: String) {
        self.opt_passes = opt_passes;
    }

    pub fn get_modificator_passes(&self) -> &[LLVMModificatorPasses] {
        &self.modificator_passes
    }

    pub fn get_compilers_configuration(&self) -> &CompilersConfiguration {
        &self.compilers_config
    }

    pub fn get_mut_compilers_configuration(&mut self) -> &mut CompilersConfiguration {
        &mut self.compilers_config
    }

    pub fn set_modificator_passes(&mut self, modificator_passes: Vec<LLVMModificatorPasses>) {
        self.modificator_passes = modificator_passes;
    }
}

impl LLVMModificatorPasses {
    pub fn raw_str_into_llvm_modificator_passes(raw: &str) -> Vec<LLVMModificatorPasses> {
        let mut passes: Vec<LLVMModificatorPasses> = Vec::with_capacity(10);

        raw.split(";").for_each(|pass| match pass {
            "loopvectorization" => passes.push(LLVMModificatorPasses::LoopVectorization),
            "loopunroll" => passes.push(LLVMModificatorPasses::LoopUnroll),
            "loopinterleaving" => passes.push(LLVMModificatorPasses::LoopInterleaving),
            "loopsimplifyvectorization" => {
                passes.push(LLVMModificatorPasses::LoopSimplifyVectorization)
            }
            "mergefunctions" => passes.push(LLVMModificatorPasses::MergeFunctions),
            _ => {
                logging::log(
                    logging::LoggingType::Warning,
                    &format!("Unknown LLVM modificator pass '{}'.", pass),
                );
            }
        });

        passes
    }
}

impl CompilersConfiguration {
    pub fn new() -> Self {
        Self {
            use_clang: true,
            use_gcc: false,
            compiler_args: Vec::with_capacity(50),
            custom_gcc: None,
            custom_clang: None,
        }
    }

    pub fn set_use_clang(&mut self, value: bool) {
        self.use_clang = value;
    }

    pub fn set_use_gcc(&mut self, value: bool) {
        self.use_gcc = value;
    }

    pub fn add_compiler_arg(&mut self, value: String) {
        self.compiler_args.push(value);
    }

    pub fn set_custom_clang(&mut self, value: PathBuf) {
        self.custom_clang = Some(value);
    }

    pub fn set_custom_gcc(&mut self, value: PathBuf) {
        self.custom_gcc = Some(value);
    }

    pub fn get_args(&self) -> &[String] {
        &self.compiler_args
    }

    pub fn get_custom_clang(&self) -> Option<&PathBuf> {
        self.custom_clang.as_ref()
    }

    pub fn get_custom_gcc(&self) -> Option<&PathBuf> {
        self.custom_gcc.as_ref()
    }

    pub fn use_clang(&self) -> bool {
        self.use_clang
    }

    pub fn use_gcc(&self) -> bool {
        self.use_gcc
    }
}
