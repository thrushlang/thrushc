use crate::core::{compiler::backends::llvm::passes::LLVMModificatorPasses, console::logging};

use inkwell::{
    OptimizationLevel,
    attributes::{Attribute, AttributeLoc},
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::TargetMachine,
    values::{
        AsValueRef, BasicValueEnum, CallSiteValue, FunctionValue, InstructionOpcode,
        InstructionValue,
    },
};
use llvm_sys::core::LLVMGetOperand;

#[derive(Debug)]
pub struct LLVMOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    target_machine: &'a TargetMachine,
    opt_level: OptimizationLevel,
    custom_passes: &'ctx str,
    modicator_passes: &'ctx [LLVMModificatorPasses],
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        target_machine: &'a TargetMachine,
        opt_level: OptimizationLevel,
        custom_passes: &'ctx str,
        modicator_passes: &'ctx [LLVMModificatorPasses],
    ) -> Self {
        Self {
            module,
            context,
            target_machine,
            opt_level,
            custom_passes,
            modicator_passes,
        }
    }
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
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
            OptimizationLevel::None => {
                let mut param_opt: LLVMParameterOptimizer =
                    LLVMParameterOptimizer::new(self.module, self.context);

                param_opt.start();
            }

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
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
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

#[derive(Debug)]
pub struct LLVMParameterOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,

    function: Option<FunctionValue<'ctx>>,
    target: Option<BasicValueEnum<'ctx>>,
    target_position: Option<u32>,
    optimizations: Option<LLVMParameterOptimizations>,
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    #[inline]
    pub fn new(module: &'a Module<'ctx>, context: &'ctx Context) -> Self {
        Self {
            module,
            context,

            function: None,
            target: None,
            target_position: None,
            optimizations: None,
        }
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    pub fn start(&mut self) {
        self.module.get_functions().for_each(|function_value| {
            self.visit_function(function_value);
        });
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    fn optimize(&mut self) {
        if let Some(optimizations) = self.optimizations {
            if optimizations.has_nocapture() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("nocapture");

                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    if let Some(target_pos) = self.target_position {
                        function.add_attribute(AttributeLoc::Param(target_pos), attribute);
                    }
                }
            }
        }
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    fn visit_function(&mut self, function: FunctionValue<'ctx>) {
        if function.get_first_basic_block().is_none() {
            return;
        }

        self.set_function(function);

        function
            .get_param_iter()
            .enumerate()
            .for_each(|(idx, parameter)| {
                self.set_target(parameter, idx as u32);
                self.set_optimizations();

                function.get_basic_block_iter().for_each(|basic_block| {
                    self.visit_basic_block(basic_block);
                });

                self.optimize();

                self.reset_optimizations();
                self.reset_target();
            });

        self.reset_function();
    }

    fn visit_basic_block(&mut self, basic_block: BasicBlock<'ctx>) {
        basic_block.get_instructions().for_each(|instruction| {
            if self.target.is_some_and(|target| target.is_pointer_value()) {
                self.visit_instruction_for_nocapture(instruction);
            }
        });
    }

    fn visit_instruction_for_nocapture(&mut self, instruction: InstructionValue<'ctx>) {
        match instruction.get_opcode() {
            InstructionOpcode::Store => {
                if let Some(store_dest) = instruction.get_operand(0) {
                    if let Some(left) = store_dest.left() {
                        if let Some(target) = self.target {
                            if left == target {
                                if let Some(opts) = self.get_optimizations() {
                                    opts.set_nocapture(false);
                                }
                            }
                        }
                    }
                }
            }
            InstructionOpcode::Return => {
                if let Some(return_value) = instruction.get_operand(0) {
                    if let Some(left) = return_value.left() {
                        if let Some(target) = self.target {
                            if left == target {
                                if let Some(opts) = self.get_optimizations() {
                                    opts.set_nocapture(false);
                                }
                            }
                        }
                    }
                }
            }
            InstructionOpcode::Call => {
                let Ok(callsite) = CallSiteValue::try_from(instruction) else {
                    return;
                };

                let called_fn: FunctionValue = callsite.get_called_fn_value();

                if self.function.is_some_and(|function| function != called_fn) {
                    self.visit_function(called_fn);
                }

                let num_args: u32 = callsite.count_arguments();
                let mut passed_index: Option<u32> = None;

                for i in 0..num_args {
                    if let Some(operand) = self::get_call_operand(callsite, i) {
                        if let Some(target) = self.target {
                            if operand.is_pointer_value() && operand.into_pointer_value() == target
                            {
                                passed_index = Some(i);
                                break;
                            }
                        }
                    }
                }

                let Some(idx) = passed_index else {
                    return;
                };

                let kind_id: u32 =
                    inkwell::attributes::Attribute::get_named_enum_kind_id("nocapture");

                let has_nocapture: bool = called_fn
                    .get_enum_attribute(inkwell::attributes::AttributeLoc::Param(idx), kind_id)
                    .is_some();

                if !has_nocapture {
                    if let Some(opts) = self.get_optimizations() {
                        opts.set_nocapture(false);
                    }
                }
            }

            _ => (),
        }
    }
}

pub fn get_call_operand<'ctx>(
    callsite: CallSiteValue<'ctx>,
    index: u32,
) -> Option<BasicValueEnum<'ctx>> {
    if index >= callsite.count_arguments() {
        return None;
    }

    let operand_ref: *mut llvm_sys::LLVMValue =
        unsafe { LLVMGetOperand(callsite.as_value_ref(), index) };

    if operand_ref.is_null() {
        None
    } else {
        unsafe { Some(BasicValueEnum::new(operand_ref)) }
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    pub fn set_optimizations(&mut self) {
        if let Some(target) = self.target {
            self.optimizations = Some(LLVMParameterOptimizations {
                nocapture: target.is_pointer_value(),
            });
        }
    }

    #[inline]
    pub fn reset_optimizations(&mut self) {
        self.optimizations = None;
    }

    #[inline]
    pub fn set_target(&mut self, target: BasicValueEnum<'ctx>, position: u32) {
        self.target = Some(target);
        self.target_position = Some(position);
    }

    #[inline]
    pub fn reset_target(&mut self) {
        self.target = None;
        self.target_position = None;
    }

    #[inline]
    pub fn set_function(&mut self, function: FunctionValue<'ctx>) {
        self.function = Some(function);
    }

    #[inline]
    pub fn reset_function(&mut self) {
        self.function = None;
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    #[inline]
    pub fn get_optimizations(&mut self) -> Option<&mut LLVMParameterOptimizations> {
        self.optimizations.as_mut()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMParameterOptimizations {
    nocapture: bool,
}

impl LLVMParameterOptimizations {
    #[inline]
    pub fn has_nocapture(&self) -> bool {
        self.nocapture
    }
}

impl LLVMParameterOptimizations {
    #[inline]
    pub fn set_nocapture(&mut self, value: bool) {
        self.nocapture = value;
    }
}
