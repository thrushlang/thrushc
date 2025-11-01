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
    CallGraphProfile,
    ForgetAllScevInLoopUnroll,
    LicmMssaNoAccForPromotionCap(u32),
    LicmMssaOptCap(u32),
}

impl LLVMModificatorPasses {
    pub fn into_llvm_modificator_passes(raw: &str) -> Vec<LLVMModificatorPasses> {
        let mut passes: Vec<LLVMModificatorPasses> = Vec::with_capacity(10);

        for pass in raw.split(';') {
            let pass = pass.trim();
            if pass.is_empty() {
                continue;
            }

            if let Some((key, value)) = pass.split_once('=') {
                let key: String = key.trim().to_lowercase();
                let value: &str = value.trim();

                match key.as_str() {
                    "licmmssaaccpromcap" => match value.parse::<u32>() {
                        Ok(cap) => {
                            passes.push(LLVMModificatorPasses::LicmMssaNoAccForPromotionCap(cap))
                        }
                        Err(_) => logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!(
                                "Invalid cap value for LicmMssaNoAccForPromotionCap: '{}'",
                                value
                            ),
                        ),
                    },

                    "licmmssaoptcap" => match value.parse::<u32>() {
                        Ok(cap) => passes.push(LLVMModificatorPasses::LicmMssaOptCap(cap)),
                        Err(_) => logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!("Invalid cap value for LicmMssaOptCap: '{}'", value),
                        ),
                    },

                    _ => logging::print_warn(
                        logging::LoggingType::Warning,
                        &format!("Unknown parameterized LLVM pass: '{}={}'", key, value),
                    ),
                }
            } else {
                match pass.to_lowercase().as_str() {
                    "loopvectorization" => passes.push(LLVMModificatorPasses::LoopVectorization),
                    "loopunroll" => passes.push(LLVMModificatorPasses::LoopUnroll),
                    "loopinterleaving" => passes.push(LLVMModificatorPasses::LoopInterleaving),
                    "loopsimplifyvectorization" => {
                        passes.push(LLVMModificatorPasses::LoopSimplifyVectorization)
                    }
                    "mergefunctions" => passes.push(LLVMModificatorPasses::MergeFunctions),
                    "callgraphprofile" => passes.push(LLVMModificatorPasses::CallGraphProfile),
                    "forgetallscevinloopunroll" => {
                        passes.push(LLVMModificatorPasses::ForgetAllScevInLoopUnroll)
                    }

                    _ => logging::print_warn(
                        logging::LoggingType::Warning,
                        &format!("Unknown LLVM modificator pass: '{}'", pass),
                    ),
                }
            }
        }

        passes
    }
}
