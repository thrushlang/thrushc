#![allow(clippy::needless_bool)]

use inkwell::debug_info::DICompileUnit;
use inkwell::debug_info::DWARFEmissionKind;
use inkwell::debug_info::DebugInfoBuilder;
use inkwell::module::Module;

use crate::core::compiler::options::CompilationUnit;
use crate::core::compiler::options::CompilerOptions;
use crate::core::constants::COMPILER_ID;

#[derive(Debug)]
pub struct LLVMDebugContext<'ctx> {
    builder: DebugInfoBuilder<'ctx>,
    unit: DICompileUnit<'ctx>,
}

impl<'ctx> LLVMDebugContext<'ctx> {
    pub fn new(module: &Module<'ctx>, options: &CompilerOptions, unit: &CompilationUnit) -> Self {
        let is_optimized: bool = if options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt()
        {
            false
        } else {
            true
        };

        let split_debug_inlining: bool = options
            .get_llvm_backend_options()
            .get_debug_config()
            .need_split_debug_inlining();

        let debug_info_for_profiling: bool = options
            .get_llvm_backend_options()
            .get_debug_config()
            .need_debug_info_for_profiling();

        let (builder, dicompileunit) = module.create_debug_info_builder(
            true,
            inkwell::debug_info::DWARFSourceLanguage::C,
            unit.get_name(),
            &format!("{}", unit.get_path().display()),
            COMPILER_ID,
            is_optimized,
            "",
            0,
            "",
            DWARFEmissionKind::Full,
            0,
            split_debug_inlining,
            debug_info_for_profiling,
            "",
            "",
        );

        Self {
            builder,
            unit: dicompileunit,
        }
    }
}

impl<'ctx> LLVMDebugContext<'ctx> {
    #[inline]
    pub fn finalize(self) {
        self.builder.finalize();
    }
}

impl<'ctx> LLVMDebugContext<'ctx> {
    #[inline]
    pub fn get_builder(&self) -> &DebugInfoBuilder<'ctx> {
        &self.builder
    }

    #[inline]
    pub fn get_unit(&self) -> &DICompileUnit<'ctx> {
        &self.unit
    }
}
