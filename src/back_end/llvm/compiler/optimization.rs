use crate::core::{compiler::backends::llvm::passes::LLVMModificatorPasses, console::logging};

use inkwell::{
    OptimizationLevel, module::Module, passes::PassBuilderOptions, targets::TargetMachine,
};

#[derive(Debug)]
pub struct LLVMOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    target_machine: &'a TargetMachine,
    opt_level: OptimizationLevel,
    custom_passes: &'ctx str,
    modicator_passes: &'ctx [LLVMModificatorPasses],
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        target_machine: &'a TargetMachine,
        opt_level: OptimizationLevel,
        custom_passes: &'ctx str,
        modicator_passes: &'ctx [LLVMModificatorPasses],
    ) -> Self {
        Self {
            module,
            target_machine,
            opt_level,
            custom_passes,
            modicator_passes,
        }
    }

    #[inline]
    pub fn optimize(&self) {
        if !self.custom_passes.is_empty() {
            if let Err(error) = self.module.run_passes(
                self.custom_passes,
                self.target_machine,
                self.create_passes_builder(),
            ) {
                logging::print_warn(
                    logging::LoggingType::Warning,
                    &format!(
                        "Some optimizations passes couldn't be performed because: '{}'.",
                        error
                    ),
                );

                return;
            }

            return;
        }

        match self.opt_level {
            OptimizationLevel::None => (),

            OptimizationLevel::Default => {
                if let Err(error) = self.module.run_passes(
                    "default<O1>",
                    self.target_machine,
                    self.create_passes_builder(),
                ) {
                    logging::print_warn(
                        logging::LoggingType::Warning,
                        &format!(
                            "Some optimizations passes couldn't be performed because: '{}'.",
                            error
                        ),
                    );
                }
            }

            OptimizationLevel::Less => {
                if let Err(error) = self.module.run_passes(
                    "default<O2>",
                    self.target_machine,
                    self.create_passes_builder(),
                ) {
                    logging::print_warn(
                        logging::LoggingType::Warning,
                        &format!(
                            "Some optimizations passes couldn't be performed because: '{}'.",
                            error
                        ),
                    );
                }
            }

            OptimizationLevel::Aggressive => {
                if let Err(error) = self.module.run_passes(
                    "default<O3>",
                    self.target_machine,
                    self.create_passes_builder(),
                ) {
                    logging::print_warn(
                        logging::LoggingType::Warning,
                        &format!(
                            "Some optimizations passes couldn't be performed because: '{:?}'.",
                            error
                        ),
                    );
                }
            }
        }
    }

    fn create_passes_builder(&self) -> PassBuilderOptions {
        let passes_builder: PassBuilderOptions = PassBuilderOptions::create();

        self.modicator_passes.iter().for_each(|pass| match pass {
            LLVMModificatorPasses::LoopVectorization => {
                passes_builder.set_loop_vectorization(true);
            }
            LLVMModificatorPasses::LoopUnroll => {
                passes_builder.set_loop_unrolling(true);
            }
            LLVMModificatorPasses::LoopInterleaving => {
                passes_builder.set_loop_interleaving(true);
            }
            LLVMModificatorPasses::LoopSimplifyVectorization => {
                passes_builder.set_loop_slp_vectorization(true);
            }
            LLVMModificatorPasses::MergeFunctions => {
                passes_builder.set_merge_functions(true);
            }
            LLVMModificatorPasses::CallGraphProfile => {
                passes_builder.set_call_graph_profile(true);
            }
            LLVMModificatorPasses::ForgetAllScevInLoopUnroll => {
                passes_builder.set_forget_all_scev_in_loop_unroll(true);
            }
            LLVMModificatorPasses::LicmMssaNoAccForPromotionCap(value) => {
                passes_builder.set_licm_mssa_no_acc_for_promotion_cap(*value);
            }
            LLVMModificatorPasses::LicmMssaOptCap(value) => {
                passes_builder.set_licm_mssa_opt_cap(*value);
            }
        });

        passes_builder
    }
}
