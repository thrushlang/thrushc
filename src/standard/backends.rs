use std::env;

use inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple};
use lld::LldFlavor;

use crate::standard::logging;

use super::misc::{Emitable, ThrushOptimization};

/* ######################################################################


    LLVM BACKEND - START


########################################################################*/

#[derive(Debug, Clone, Copy)]
pub enum LLVMExecutableFlavor {
    Wasm,
    MachO,
    Elf,
    Coff,
}

#[derive(Debug, Clone, Copy)]
pub enum LLVMModificatorPasses {
    LoopVectorization,
    LoopUnroll,
    LoopInterleaving,
    LoopSimplifyVectorization,
    MergeFunctions,
}

#[derive(Debug)]
pub struct LLVMBackend {
    target_cpu: String,
    target_triple: TargetTriple,
    executable_flavor: LLVMExecutableFlavor,
    optimization: ThrushOptimization,
    emit: Vec<Emitable>,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    modificator_passes: Vec<LLVMModificatorPasses>,
    opt_passes: String,
    linker_flags: String,
}

impl LLVMBackend {
    pub fn new() -> Self {
        Self {
            target_cpu: String::with_capacity(100),
            target_triple: TargetMachine::get_default_triple(),
            executable_flavor: LLVMExecutableFlavor::default(),
            optimization: ThrushOptimization::None,
            emit: Vec::with_capacity(10),
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            modificator_passes: Vec::with_capacity(10),
            opt_passes: String::with_capacity(100),
            linker_flags: String::with_capacity(100),
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

    pub fn get_linker_flags(&self) -> &str {
        self.linker_flags.as_str()
    }

    pub fn contains_emitable(&self, emit: Emitable) -> bool {
        self.emit.contains(&emit)
    }

    pub fn set_linker_flags(&mut self, lk_flags: String) {
        self.linker_flags = lk_flags;
    }

    pub fn get_target_cpu(&self) -> &str {
        self.target_cpu.as_str()
    }

    pub fn set_target_cpu(&mut self, target_cpu: String) {
        self.target_cpu = target_cpu;
    }

    pub fn set_executable_flavor(&mut self, flavor: LLVMExecutableFlavor) {
        self.executable_flavor = flavor;
    }

    pub fn get_executable_flavor(&self) -> LLVMExecutableFlavor {
        self.executable_flavor
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

impl LLVMExecutableFlavor {
    pub fn into_llvm_linker_flavor(self) -> LldFlavor {
        match self {
            LLVMExecutableFlavor::Wasm => LldFlavor::Wasm,
            LLVMExecutableFlavor::MachO => LldFlavor::MachO,
            LLVMExecutableFlavor::Elf => LldFlavor::Elf,
            LLVMExecutableFlavor::Coff => LldFlavor::Coff,
        }
    }

    pub fn raw_str_into_llvm_executable_flavor(raw: &str) -> LLVMExecutableFlavor {
        match raw {
            "wasm" => LLVMExecutableFlavor::Wasm,
            "mach0" => LLVMExecutableFlavor::MachO,
            "elf" => LLVMExecutableFlavor::Elf,
            "coff" => LLVMExecutableFlavor::Coff,
            _ => {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!(
                        "Incompatible LLVM executable flavor '{}' for compilation.",
                        raw
                    ),
                );

                unreachable!()
            }
        }
    }

    pub fn default() -> Self {
        match env::consts::OS {
            "windows" => LLVMExecutableFlavor::Coff,
            "linux" => LLVMExecutableFlavor::Elf,
            _ => {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!(
                        "Incompatible host operating system '{}' for compilation.",
                        env::consts::OS
                    ),
                );

                unreachable!()
            }
        }
    }
}

/* ######################################################################


    LLVM BACKEND - END


########################################################################*/
