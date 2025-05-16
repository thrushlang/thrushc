use inkwell::{
    OptimizationLevel, module::Module, passes::PassBuilderOptions, targets::TargetMachine,
};

use crate::standard::logging;

pub struct Optimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    target_machine: &'a TargetMachine,
    opt_level: OptimizationLevel,
}

impl<'a, 'ctx> Optimizer<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        target_machine: &'a TargetMachine,
        opt_level: OptimizationLevel,
    ) -> Self {
        Self {
            module,
            target_machine,
            opt_level,
        }
    }

    pub fn optimize(&self) {
        match self.opt_level {
            OptimizationLevel::None => (),
            OptimizationLevel::Default => {
                if let Err(error) = self.module.run_passes(
                    "default<O1>",
                    self.target_machine,
                    self.create_passes_builder(),
                ) {
                    logging::log(
                        logging::LoggingType::Warning,
                        &format!(
                            "Optimization passes could not be performed on the file: '{:?}'.",
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
                    logging::log(
                        logging::LoggingType::Warning,
                        &format!(
                            "Optimization passes could not be performed on the file: '{:?}'.",
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
                    logging::log(
                        logging::LoggingType::Warning,
                        &format!(
                            "Optimization passes could not be performed on the file: '{:?}'.",
                            error
                        ),
                    );
                }
            }
        }
    }

    fn create_passes_builder(&self) -> PassBuilderOptions {
        PassBuilderOptions::create()
    }
}
