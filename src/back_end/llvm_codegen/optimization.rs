use crate::core::compiler::backends::llvm::passes::LLVMModificatorPasses;
use crate::core::compiler::options::CompilerOptions;
use crate::core::compiler::options::ThrushOptimization;
use crate::core::console::logging;

use inkwell::attributes::Attribute;
use inkwell::attributes::AttributeLoc;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::TargetMachine;
use inkwell::types::BasicTypeEnum;
use inkwell::values::AsValueRef;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::FunctionValue;
use inkwell::values::InstructionOpcode;
use inkwell::values::InstructionValue;

#[derive(Debug)]
pub struct LLVMOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    machine: &'a TargetMachine,
    flags: LLVMOptimizerFlags,
    passes: LLVMOptimizerPasses<'ctx>,
    opt_level: ThrushOptimization,
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        machine: &'a TargetMachine,
        flags: LLVMOptimizerFlags,
        passes: LLVMOptimizerPasses<'ctx>,
        opt_level: ThrushOptimization,
    ) -> Self {
        Self {
            module,
            context,
            machine,
            flags,
            passes,
            opt_level,
        }
    }
}

impl LLVMOptimizer<'_, '_> {
    #[inline]
    pub fn optimize(&self) {
        let custom_passes: &str = self.get_passes().get_llvm_custom_passes();
        let machine: &TargetMachine = self.get_machine();
        let options: PassBuilderOptions = self.create_passes_builder();

        if !custom_passes.is_empty() {
            if let Err(error) = self
                .get_module()
                .run_passes(custom_passes, machine, options)
            {
                logging::print_warn(
                    logging::LoggingType::Warning,
                    &format!(
                        "Some optimizations passes couldn't be performed because: '{}'.",
                        error
                    ),
                );
            }
        } else {
            match self.opt_level {
                ThrushOptimization::None => {
                    if !self.get_flags().get_disable_default_opt() {
                        let mut param_opt: LLVMParameterOptimizer =
                            LLVMParameterOptimizer::new(self.get_module(), self.get_context());

                        param_opt.start();
                    }
                }

                ThrushOptimization::Low => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<O1>", machine, options)
                    {
                        logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!(
                                "Some optimizations passes couldn't be performed because: '{}'.",
                                error
                            ),
                        );
                    }
                }

                ThrushOptimization::Mid => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<O2>", machine, options)
                    {
                        logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!(
                                "Some optimizations passes couldn't be performed because: '{}'.",
                                error
                            ),
                        );
                    }
                }

                ThrushOptimization::High => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<O3>", machine, options)
                    {
                        logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!(
                                "Some optimizations passes couldn't be performed because: '{:?}'.",
                                error
                            ),
                        );
                    }
                }

                ThrushOptimization::Size => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<Os>", machine, options)
                    {
                        logging::print_warn(
                            logging::LoggingType::Warning,
                            &format!(
                                "Some optimizations passes couldn't be performed because: '{:?}'.",
                                error
                            ),
                        );
                    }
                }

                ThrushOptimization::Zize => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<Oz>", machine, options)
                    {
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
}

impl LLVMOptimizer<'_, '_> {
    fn create_passes_builder(&self) -> PassBuilderOptions {
        let passes_builder: PassBuilderOptions = PassBuilderOptions::create();

        self.get_passes()
            .get_llvm_modificator_passes()
            .iter()
            .for_each(|pass| match pass {
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

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
    #[inline]
    pub fn get_module(&self) -> &Module<'ctx> {
        self.module
    }

    #[inline]
    pub fn get_context(&self) -> &'ctx Context {
        self.context
    }

    #[inline]
    pub fn get_flags(&self) -> &LLVMOptimizerFlags {
        &self.flags
    }

    #[inline]
    pub fn get_passes(&self) -> &LLVMOptimizerPasses<'_> {
        &self.passes
    }

    #[inline]
    pub fn get_machine(&self) -> &TargetMachine {
        self.machine
    }
}

impl LLVMOptimizer<'_, '_> {
    pub fn is_optimizable(
        entity: LLVMOptimizerOptimizableEntity,
        options: &CompilerOptions,
    ) -> bool {
        let before: bool = (!options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt())
            || options
                .get_llvm_backend_options()
                .get_optimization()
                .is_high_opt();

        match entity {
            LLVMOptimizerOptimizableEntity::Function(value) => {
                let parameters_types: Vec<BasicTypeEnum> = value
                    .get_param_iter()
                    .map(|param| param.get_type())
                    .collect();

                if parameters_types
                    .iter()
                    .any(|parameter_type| parameter_type.is_pointer_type())
                    && before
                {
                    return true;
                }

                false
            }
        }
    }

    pub fn is_optimizable_module(llvm_module: &Module, options: &CompilerOptions) -> bool {
        let before: bool = (!options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt())
            || options
                .get_llvm_backend_options()
                .get_optimization()
                .is_high_opt();

        if !options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt()
        {
            llvm_module
                .get_functions()
                .filter(|function| function.get_first_basic_block().is_some())
                .any(|function| {
                    function
                        .get_param_iter()
                        .any(|param_ty| param_ty.is_pointer_value())
                })
        } else {
            llvm_module
                .get_functions()
                .filter(|function| function.get_first_basic_block().is_some())
                .any(|function| {
                    function
                        .get_param_iter()
                        .any(|param_ty| param_ty.is_pointer_value())
                })
                || before
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LLVMOptimizerOptimizableEntity<'ctx> {
    Function(FunctionValue<'ctx>),
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMOptimizerPasses<'ctx> {
    custom_passes: &'ctx str,
    modicator_passes: &'ctx [LLVMModificatorPasses],
}

impl<'ctx> LLVMOptimizerPasses<'ctx> {
    pub fn new(custom_passes: &'ctx str, modicator_passes: &'ctx [LLVMModificatorPasses]) -> Self {
        Self {
            custom_passes,
            modicator_passes,
        }
    }
}

impl<'ctx> LLVMOptimizerPasses<'ctx> {
    #[inline]
    pub fn get_llvm_custom_passes(&self) -> &str {
        self.custom_passes
    }

    #[inline]
    pub fn get_llvm_modificator_passes(&self) -> &[LLVMModificatorPasses] {
        self.modicator_passes
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMOptimizerFlags {
    disable_default_opt: bool,
}

impl LLVMOptimizerFlags {
    #[inline]
    pub fn new(disable_default_opt: bool) -> Self {
        Self {
            disable_default_opt,
        }
    }
}

impl LLVMOptimizerFlags {
    #[inline]
    pub fn get_disable_default_opt(&self) -> bool {
        self.disable_default_opt
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
            self.visit_function_once(function_value);
        });
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    fn optimize(&mut self) {
        if let Some(optimizations) = self.get_optimizations() {
            if optimizations.has_deferenceable() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("dereferenceable");

                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 1);

                if let Some(function) = self.function {
                    if let Some(target_pos) = self.target_position {
                        function.add_attribute(AttributeLoc::Param(target_pos), attribute);
                    }
                }
            }

            if optimizations.has_noundef() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("noundef");
                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    if let Some(target_pos) = self.target_position {
                        function.add_attribute(AttributeLoc::Param(target_pos), attribute);
                    }
                }
            }

            if optimizations.has_align() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("align");
                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 1);

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
    fn visit_function_once(&mut self, function: FunctionValue<'ctx>) {
        if function.get_first_basic_block().is_none() {
            return;
        }

        self.set_function(function);

        function
            .get_param_iter()
            .enumerate()
            .for_each(|(idx, parameter)| {
                self.set_target(parameter, idx as u32);
                self.set_optimizations(function);

                function.get_basic_block_iter().for_each(|basic_block| {
                    self.visit_basic_block_once(basic_block);
                });

                self.optimize();

                self.reset_optimizations();
                self.reset_target();
            });

        self.reset_function();
    }

    fn visit_basic_block_once(&mut self, basic_block: BasicBlock<'ctx>) {
        basic_block.get_instructions().for_each(|instruction| {
            self.visit_instruction_once(instruction);
        });
    }

    fn visit_instruction_once(&mut self, instruction: InstructionValue<'ctx>) {
        if instruction.get_opcode() == InstructionOpcode::Call {
            let callsite: CallSiteValue = unsafe { CallSiteValue::new(instruction.as_value_ref()) };

            let callfn: FunctionValue = callsite.get_called_fn_value();

            if !callsite.is_tail_call() && self.function.is_some_and(|function| function == callfn)
            {
                callsite.set_tail_call(true);
            }
        }
    }
}

impl<'a, 'ctx> LLVMParameterOptimizer<'a, 'ctx> {
    pub fn set_optimizations(&mut self, function: FunctionValue<'ctx>) {
        if let Some(target) = self.target {
            self.optimizations = Some(LLVMParameterOptimizations {
                deferenceable: target.is_pointer_value(),
                noundef: !function.get_type().is_var_arg(),
                align: target.is_pointer_value() && !function.get_type().is_var_arg(),
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
    pub fn get_optimizations(&self) -> Option<&LLVMParameterOptimizations> {
        self.optimizations.as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMParameterOptimizations {
    deferenceable: bool,
    noundef: bool,
    align: bool,
}

impl LLVMParameterOptimizations {
    #[inline]
    pub fn has_deferenceable(&self) -> bool {
        self.deferenceable
    }

    #[inline]
    pub fn has_noundef(&self) -> bool {
        self.noundef
    }

    #[inline]
    pub fn has_align(&self) -> bool {
        self.align
    }
}
