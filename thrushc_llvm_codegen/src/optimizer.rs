use inkwell::attributes::Attribute;
use inkwell::attributes::AttributeLoc;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::TargetMachine;
use inkwell::values::AsValueRef;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::FunctionValue;
use inkwell::values::InstructionOpcode;
use inkwell::values::InstructionValue;

use thrushc_options::CompilerOptions;
use thrushc_options::ThrushOptimization;
use thrushc_options::backends::llvm::Sanitizer;
use thrushc_options::backends::llvm::passes::LLVMModificatorPasses;

#[derive(Debug)]
pub struct LLVMOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    machine: &'a TargetMachine,
    config: LLVMOptimizationConfig,
    flags: LLVMOptimizerFlags,
    passes: LLVMOptimizerPasses<'ctx>,
}

impl<'a, 'ctx> LLVMOptimizer<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        machine: &'a TargetMachine,
        config: LLVMOptimizationConfig,
        flags: LLVMOptimizerFlags,
        passes: LLVMOptimizerPasses<'ctx>,
    ) -> Self {
        Self {
            module,
            context,
            machine,
            config,
            flags,
            passes,
        }
    }
}

impl LLVMOptimizer<'_, '_> {
    #[inline]
    pub fn optimize(&self) {
        let custom_passes: &str = self.get_passes().get_llvm_custom_passes();
        let machine: &TargetMachine = self.get_machine();
        let options: PassBuilderOptions = self.create_passes_builder();
        let config: LLVMOptimizationConfig = self.get_config();

        let module: &Module = self.get_module();
        let context: &Context = self.get_context();

        LLVMSanitizer::new(module, context, config).run();

        if !self.get_flags().get_disable_default_opt()
            && !config.get_compiler_optimization().is_high_opt()
        {
            LLVMParameterOptimizer::new(module, context).run();
            LLVMFunctionOptimizer::new(module, context).run();
        }

        if !custom_passes.is_empty() {
            if let Err(error) = self
                .get_module()
                .run_passes(custom_passes, machine, options)
            {
                thrushc_logging::print_warn(
                    thrushc_logging::LoggingType::Warning,
                    &format!(
                        "Some optimizations passes couldn't be performed because: '{}'.",
                        error
                    ),
                );
            }
        } else {
            match config.get_compiler_optimization() {
                ThrushOptimization::None => {}

                ThrushOptimization::Low => {
                    if let Err(error) =
                        self.get_module()
                            .run_passes("default<O1>", machine, options)
                    {
                        thrushc_logging::print_warn(
                            thrushc_logging::LoggingType::Warning,
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
                        thrushc_logging::print_warn(
                            thrushc_logging::LoggingType::Warning,
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
                        thrushc_logging::print_warn(
                            thrushc_logging::LoggingType::Warning,
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
                        thrushc_logging::print_warn(
                            thrushc_logging::LoggingType::Warning,
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
                        thrushc_logging::print_warn(
                            thrushc_logging::LoggingType::Warning,
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

    #[inline]
    pub fn get_config(&self) -> LLVMOptimizationConfig {
        self.config
    }
}

impl LLVMOptimizer<'_, '_> {
    #[inline]
    pub fn is_optimizable(options: &CompilerOptions) -> bool {
        (!options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt())
            || options
                .get_llvm_backend_options()
                .get_optimization()
                .is_high_opt()
    }
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

#[derive(Debug, Clone, Copy)]
pub struct LLVMOptimizationConfig {
    compiler_optimization: ThrushOptimization,
    sanitizer: Sanitizer,
}

impl LLVMOptimizationConfig {
    #[inline]
    pub fn new(compiler_optimization: ThrushOptimization, sanitizer: Sanitizer) -> Self {
        Self {
            compiler_optimization,
            sanitizer,
        }
    }
}

impl LLVMOptimizationConfig {
    #[inline]
    pub fn get_compiler_optimization(&self) -> ThrushOptimization {
        self.compiler_optimization
    }

    #[inline]
    pub fn get_sanitizer(&self) -> Sanitizer {
        self.sanitizer
    }
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
pub struct LLVMFunctionOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,

    function: Option<FunctionValue<'ctx>>,
    optimizations: Option<LLVMFunctionOptimizations>,
}

impl<'a, 'ctx> LLVMFunctionOptimizer<'a, 'ctx> {
    #[inline]
    pub fn new(module: &'a Module<'ctx>, context: &'ctx Context) -> Self {
        Self {
            module,
            context,

            function: None,
            optimizations: None,
        }
    }
}

impl<'a, 'ctx> LLVMFunctionOptimizer<'a, 'ctx> {
    pub fn run(&mut self) {
        self.module.get_functions().for_each(|function| {
            self.visit_function_once(function);
        });
    }
}

impl<'a, 'ctx> LLVMFunctionOptimizer<'a, 'ctx> {
    fn visit_function_once(&mut self, function: FunctionValue<'ctx>) {
        self.set_function(function);

        if function.get_first_basic_block().is_none() {
            self.reset_function();
            return;
        } else {
            let mut optimizations: LLVMFunctionOptimizations = LLVMFunctionOptimizations::new();

            const MAX_OPT_INSTRUCTIONS_LEN: usize = 5;
            const CONSIDERABLE_BASIC_BLOCKS_LEN: usize = 5;
            const CONSIDERABLE_INSTRUCTIONS_LEN: usize = 250;

            let blocks_count: usize = function.get_basic_block_iter().count();
            let instructions_count: usize = function
                .get_basic_block_iter()
                .map(|basic_block| basic_block.get_instructions().count())
                .sum();

            let applicable_norecurse: bool = function
                .get_basic_block_iter()
                .flat_map(|bb| bb.get_instructions())
                .filter(|instr| instr.get_opcode() == InstructionOpcode::Call)
                .any(|instr| {
                    let callsite: CallSiteValue =
                        unsafe { CallSiteValue::new(instr.as_value_ref()) };
                    let called: FunctionValue = callsite.get_called_fn_value();

                    self.function.is_some_and(|current| current == called)
                });

            if MAX_OPT_INSTRUCTIONS_LEN > instructions_count {
                optimizations.set_inlinehint(true);
            } else if blocks_count >= CONSIDERABLE_BASIC_BLOCKS_LEN
                && instructions_count >= CONSIDERABLE_INSTRUCTIONS_LEN
            {
                optimizations.set_optsize(true);
            }

            if !applicable_norecurse {
                optimizations.set_nocurse(true);
            }

            self.set_optimizations(optimizations);
            self.optimize_function();
            self.reset_optimizations_state();
        }

        self.reset_function();
    }
}

impl<'a, 'ctx> LLVMFunctionOptimizer<'a, 'ctx> {
    fn optimize_function(&mut self) {
        if let Some(optimizations) = self.get_optimizations() {
            if optimizations.has_inlinehint() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("inlinehint");

                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, attribute);
                }
            }

            if optimizations.has_norecurse() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("norecurse");
                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, attribute);
                }
            }

            if optimizations.has_optsize() {
                let optsize_id: u32 = Attribute::get_named_enum_kind_id("optsize");
                let minsize_id: u32 = Attribute::get_named_enum_kind_id("minsize");

                let optsize: Attribute = self.context.create_enum_attribute(optsize_id, 0);
                let minsize: Attribute = self.context.create_enum_attribute(minsize_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, optsize);
                    function.add_attribute(AttributeLoc::Function, minsize);
                }
            }

            if optimizations.has_nounwind() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("nounwind");
                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, attribute);
                }
            }

            if optimizations.has_uwtable() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("uwtable");
                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 1);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, attribute);
                }
            }

            if optimizations.has_sspstrong() {
                let kind_id: u32 = Attribute::get_named_enum_kind_id("sspstrong");

                let attribute: Attribute = self.context.create_enum_attribute(kind_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, attribute);
                }
            }
        }
    }
}

impl<'a, 'ctx> LLVMFunctionOptimizer<'a, 'ctx> {
    #[inline]
    fn set_function(&mut self, function: FunctionValue<'ctx>) {
        self.function = Some(function);
    }

    #[inline]
    fn reset_function(&mut self) {
        self.function = None;
    }

    #[inline]
    fn set_optimizations(&mut self, optimizations: LLVMFunctionOptimizations) {
        self.optimizations = Some(optimizations);
    }

    #[inline]
    fn reset_optimizations_state(&mut self) {
        self.optimizations = None;
    }
}

impl LLVMFunctionOptimizer<'_, '_> {
    #[inline]
    fn get_optimizations(&self) -> Option<LLVMFunctionOptimizations> {
        self.optimizations
    }
}

#[derive(Debug, Clone, Copy)]
struct LLVMFunctionOptimizations {
    norecurse: bool,
    nounwind: bool,
    inlinehint: bool,
    optsize: bool,
    uwtable: bool,
    sspstrong: bool,
}

impl LLVMFunctionOptimizations {
    #[inline]
    fn new() -> Self {
        Self {
            norecurse: false,
            nounwind: true,
            inlinehint: false,
            optsize: false,
            uwtable: true,
            sspstrong: true,
        }
    }
}

impl LLVMFunctionOptimizations {
    #[inline]
    pub fn set_nocurse(&mut self, value: bool) {
        self.norecurse = value;
    }

    #[inline]
    pub fn set_inlinehint(&mut self, value: bool) {
        self.inlinehint = value;
    }

    #[inline]
    pub fn set_optsize(&mut self, value: bool) {
        self.optsize = value;
    }
}

impl LLVMFunctionOptimizations {
    #[inline]
    pub fn has_norecurse(&self) -> bool {
        self.norecurse
    }

    #[inline]
    pub fn has_nounwind(&self) -> bool {
        self.nounwind
    }

    #[inline]
    pub fn has_inlinehint(&self) -> bool {
        self.inlinehint
    }

    #[inline]
    pub fn has_optsize(&self) -> bool {
        self.optsize
    }

    #[inline]
    pub fn has_uwtable(&self) -> bool {
        self.uwtable
    }

    #[inline]
    pub fn has_sspstrong(&self) -> bool {
        self.sspstrong
    }
}

#[derive(Debug)]
pub struct LLVMSanitizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,

    function: Option<FunctionValue<'ctx>>,
    optimization: Option<LLVMSanitizerOptimization>,
    config: LLVMOptimizationConfig,
}

impl<'a, 'ctx> LLVMSanitizer<'a, 'ctx> {
    #[inline]
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        config: LLVMOptimizationConfig,
    ) -> Self {
        Self {
            module,
            context,

            function: None,
            optimization: None,
            config,
        }
    }
}

impl<'a, 'ctx> LLVMSanitizer<'a, 'ctx> {
    pub fn run(&mut self) {
        let config: LLVMOptimizationConfig = self.get_config();
        let optimization: LLVMSanitizerOptimization = LLVMSanitizerOptimization::new(config);

        if optimization.is_neither() {
            return;
        }

        self.module.get_functions().for_each(|function| {
            self.visit_function_once(function);
        });
    }
}

impl<'a, 'ctx> LLVMSanitizer<'a, 'ctx> {
    fn visit_function_once(&mut self, function: FunctionValue<'ctx>) {
        self.set_function(function);

        let config: LLVMOptimizationConfig = self.get_config();
        let optimization: LLVMSanitizerOptimization = LLVMSanitizerOptimization::new(config);

        self.set_optimizations(optimization);
        self.apply();
        self.reset_optimizations_state();

        self.reset_function();
    }
}

impl<'a, 'ctx> LLVMSanitizer<'a, 'ctx> {
    fn apply(&mut self) {
        if let Some(optimizations) = self.get_optimizations() {
            if optimizations.has_sanitize_address() {
                let sanitize_address_id: u32 =
                    Attribute::get_named_enum_kind_id("sanitize_address");

                let sanitize_address: Attribute =
                    self.context.create_enum_attribute(sanitize_address_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, sanitize_address);
                }
            }

            if optimizations.has_sanitize_memory() {
                let sanitize_memory_id: u32 = Attribute::get_named_enum_kind_id("sanitize_memory");
                let sanitize_memory: Attribute =
                    self.context.create_enum_attribute(sanitize_memory_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, sanitize_memory);
                }
            }

            if optimizations.has_sanitize_thread() {
                let sanitize_thread_id: u32 = Attribute::get_named_enum_kind_id("sanitize_thread");
                let sanitize_thread: Attribute =
                    self.context.create_enum_attribute(sanitize_thread_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, sanitize_thread);
                }
            }

            if optimizations.has_sanitize_hwaddress() {
                let sanitize_hwaddress_id: u32 =
                    Attribute::get_named_enum_kind_id("sanitize_hwaddress");

                let sanitize_hwaddress: Attribute =
                    self.context.create_enum_attribute(sanitize_hwaddress_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, sanitize_hwaddress);
                }
            }

            if optimizations.has_sanitize_memtag() {
                let sanitize_memtag_id: u32 = Attribute::get_named_enum_kind_id("sanitize_memtag");
                let sanitize_memtag: Attribute =
                    self.context.create_enum_attribute(sanitize_memtag_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, sanitize_memtag);
                }
            }

            if optimizations.has_nosanitize_bounds() {
                let nosanitize_bounds_id: u32 =
                    Attribute::get_named_enum_kind_id("nosanitize_bounds");
                let nosanitize_bounds: Attribute =
                    self.context.create_enum_attribute(nosanitize_bounds_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, nosanitize_bounds);
                }
            }

            if optimizations.has_nosanitize_coverage() {
                let nosanitize_coverage_id: u32 =
                    Attribute::get_named_enum_kind_id("nosanitize_coverage");
                let nosanitize_coverage: Attribute = self
                    .context
                    .create_enum_attribute(nosanitize_coverage_id, 0);

                if let Some(function) = self.function {
                    function.add_attribute(AttributeLoc::Function, nosanitize_coverage);
                }
            }
        }
    }
}

impl<'a, 'ctx> LLVMSanitizer<'a, 'ctx> {
    #[inline]
    fn set_function(&mut self, function: FunctionValue<'ctx>) {
        self.function = Some(function);
    }

    #[inline]
    fn reset_function(&mut self) {
        self.function = None;
    }

    #[inline]
    fn set_optimizations(&mut self, optimization: LLVMSanitizerOptimization) {
        self.optimization = Some(optimization);
    }

    #[inline]
    fn reset_optimizations_state(&mut self) {
        self.optimization = None;
    }
}

impl LLVMSanitizer<'_, '_> {
    #[inline]
    fn get_optimizations(&self) -> Option<LLVMSanitizerOptimization> {
        self.optimization
    }

    #[inline]
    fn get_config(&self) -> LLVMOptimizationConfig {
        self.config
    }
}

#[derive(Debug, Clone, Copy)]
struct LLVMSanitizerOptimization {
    sanitize_address: bool,
    sanitize_memory: bool,
    sanitize_thread: bool,
    sanitize_hwaddress: bool,
    sanitize_memtag: bool,
    nosanitize_bounds: bool,
    nosanitize_coverage: bool,
}

impl LLVMSanitizerOptimization {
    #[inline]
    fn new(config: LLVMOptimizationConfig) -> Self {
        let is_sanitize_address_enabled: bool = config.get_sanitizer().is_address();
        let is_sanitize_memory_enabled: bool = config.get_sanitizer().is_memory();
        let is_sanitize_thread_enabled: bool = config.get_sanitizer().is_thread();
        let is_sanitize_hwaddres_enabled: bool = config.get_sanitizer().is_hwaddress();
        let is_sanitize_memtag_enabled: bool = config.get_sanitizer().is_memtag();

        let (nosanitize_bounds, nosanitize_coverage) = match config.get_sanitizer() {
            Sanitizer::Address(config) => (
                config.has_nosanitize_bounds(),
                config.has_nosanitize_coverage(),
            ),
            Sanitizer::Hwaddress(config) => (
                config.has_nosanitize_bounds(),
                config.has_nosanitize_coverage(),
            ),
            Sanitizer::Memory(config) => (
                config.has_nosanitize_bounds(),
                config.has_nosanitize_coverage(),
            ),
            Sanitizer::Memtag(config) => (
                config.has_nosanitize_bounds(),
                config.has_nosanitize_coverage(),
            ),
            Sanitizer::Thread(config) => (
                config.has_nosanitize_bounds(),
                config.has_nosanitize_coverage(),
            ),
            _ => (false, false),
        };

        Self {
            sanitize_address: is_sanitize_address_enabled,
            sanitize_memory: is_sanitize_memory_enabled,
            sanitize_thread: is_sanitize_thread_enabled,
            sanitize_hwaddress: is_sanitize_hwaddres_enabled,
            sanitize_memtag: is_sanitize_memtag_enabled,
            nosanitize_bounds,
            nosanitize_coverage,
        }
    }
}

impl LLVMSanitizerOptimization {
    #[inline]
    pub fn has_sanitize_address(&self) -> bool {
        self.sanitize_address
    }

    #[inline]
    pub fn has_sanitize_memory(&self) -> bool {
        self.sanitize_memory
    }

    #[inline]
    pub fn has_sanitize_thread(&self) -> bool {
        self.sanitize_thread
    }

    #[inline]
    pub fn has_sanitize_hwaddress(&self) -> bool {
        self.sanitize_hwaddress
    }

    #[inline]
    pub fn has_sanitize_memtag(&self) -> bool {
        self.sanitize_memtag
    }

    #[inline]
    pub fn has_nosanitize_bounds(&self) -> bool {
        self.nosanitize_bounds
    }

    #[inline]
    pub fn has_nosanitize_coverage(&self) -> bool {
        self.nosanitize_coverage
    }

    #[inline]
    pub fn is_neither(&self) -> bool {
        !(self.sanitize_address
            || self.sanitize_memory
            || self.sanitize_thread
            || self.sanitize_hwaddress
            || self.sanitize_memtag)
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
    pub fn run(&mut self) {
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
            let called: FunctionValue = callsite.get_called_fn_value();

            if !callsite.is_tail_call() && self.function.is_some_and(|current| current == called) {
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
