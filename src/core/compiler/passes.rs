use crate::core::console::logging;

/* ######################################################################


    LLVM BACKEND PASSES - START


########################################################################*/

#[derive(Debug, Clone, Copy)]
pub enum LLVMModificatorPasses {
    LoopVectorization,
    LoopUnroll,
    LoopInterleaving,
    LoopSimplifyVectorization,
    MergeFunctions,
}

impl LLVMModificatorPasses {
    pub fn into_llvm_modificator_passes(raw: &str) -> Vec<LLVMModificatorPasses> {
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
                logging::print_warn(
                    logging::LoggingType::Warning,
                    &format!(
                        "Unknown LLVM modificator pass provided to LLVM Optimizator '{}'.",
                        pass
                    ),
                );
            }
        });

        passes
    }
}
