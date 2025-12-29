#![allow(clippy::needless_bool)]

use std::path::PathBuf;

use inkwell::debug_info::AsDIScope;
use inkwell::debug_info::DICompileUnit;
use inkwell::debug_info::DIFile;
use inkwell::debug_info::DIFlagsConstants;
use inkwell::debug_info::DISubroutineType;
use inkwell::debug_info::DIType;
use inkwell::debug_info::DWARFEmissionKind;
use inkwell::debug_info::DebugInfoBuilder;
use inkwell::module::Module;
use inkwell::targets::TargetData;
use inkwell::targets::TargetMachine;
use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;

use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::optimization::LLVMOptimizer;
use crate::back_end::llvm_codegen::optimization::LLVMOptimizerOptimizableEntity;
use crate::back_end::llvm_codegen::typegeneration;
use crate::back_end::llvm_codegen::types::repr::LLVMDBGFunction;
use crate::back_end::llvm_codegen::types::traits::LLVMDBGFunctionExtensions;
use crate::core::compiler::options::CompilationUnit;
use crate::core::compiler::options::CompilerOptions;
use crate::core::constants::COMPILER_ID;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::front_end::typesystem::types::Type;

#[derive(Debug)]
pub struct LLVMDebugContext<'a, 'ctx> {
    builder: DebugInfoBuilder<'ctx>,
    unit: DICompileUnit<'ctx>,
    target_machine: &'a TargetMachine,
    diagnostician: Diagnostician,
}

impl<'a, 'ctx> LLVMDebugContext<'a, 'ctx> {
    pub fn new(
        module: &Module<'ctx>,
        target_machine: &'a TargetMachine,
        options: &CompilerOptions,
        unit: &CompilationUnit,
    ) -> Self {
        let is_optimized: bool = (!options.omit_default_optimizations()
            && options
                .get_llvm_backend_options()
                .get_optimization()
                .is_none_opt())
            || options
                .get_llvm_backend_options()
                .get_optimization()
                .is_high_opt();

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
            target_machine,
            diagnostician: Diagnostician::new(unit, options),
        }
    }
}

impl<'a, 'ctx> LLVMDebugContext<'a, 'ctx> {
    #[inline]
    pub fn finalize(&self) {
        self.builder.finalize();
    }
}

impl<'a, 'ctx> LLVMDebugContext<'a, 'ctx> {
    pub fn dispatch_function_debug_data(
        &mut self,
        function: &LLVMDBGFunction<'ctx>,
        context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ) {
        let value: FunctionValue<'_> = function.get_value();
        let name: &str = function.get_name();
        let return_type: &Type = function.get_return_type();
        let parameter_types: &[Type] = function.get_parameters_types();
        let span: Span = function.get_span();
        let line: u32 = u32::try_from(span.get_line()).unwrap_or_else(|_| {
            abort::abort_codegen(
                context,
                "Failed to parse the code location!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        });

        let llvm_return_type: BasicTypeEnum<'_> =
            typegeneration::compile_from(context, return_type);

        let llvm_parameter_types: Vec<BasicTypeEnum<'_>> = parameter_types
            .iter()
            .map(|parameter_type| typegeneration::compile_from(context, parameter_type))
            .collect();

        let mut dbg_parameter_types: Vec<DIType<'_>> =
            Vec::with_capacity(llvm_parameter_types.len());

        for (parameter_type, llvm_parameter_type) in
            parameter_types.iter().zip(llvm_parameter_types.iter())
        {
            let ty: DIType<'_> =
                typegeneration::compile_as_dbg_type(self, parameter_type, *llvm_parameter_type);

            dbg_parameter_types.push(ty);
        }

        let dbg_return_type: DIType =
            typegeneration::compile_as_dbg_type(self, return_type, llvm_return_type);

        let subroutine_type: DISubroutineType<'_> = self.get_builder().create_subroutine_type(
            self.get_unit().get_file(),
            Some(dbg_return_type),
            &dbg_parameter_types,
            DIFlagsConstants::PUBLIC,
        );

        let is_optimized: bool = LLVMOptimizer::is_optimizable(
            LLVMOptimizerOptimizableEntity::Function(value),
            context.get_compiler_options(),
        );

        let file: DIFile<'_> = self.get_unit().get_file();

        let function_dbg_personality = self.get_builder().create_function(
            file.as_debug_info_scope(),
            name,
            None,
            file,
            line,
            subroutine_type,
            function.is_local(),
            function.is_definition(),
            0,
            inkwell::debug_info::DIFlagsConstants::PUBLIC,
            is_optimized,
        );

        value.set_subprogram(function_dbg_personality);
    }
}

impl<'a, 'ctx> LLVMDebugContext<'a, 'ctx> {
    #[inline]
    pub fn get_builder(&self) -> &DebugInfoBuilder<'ctx> {
        &self.builder
    }

    #[inline]
    pub fn get_unit(&self) -> &DICompileUnit<'ctx> {
        &self.unit
    }

    #[inline]
    pub fn get_target_data(&self) -> TargetData {
        self.target_machine.get_target_data()
    }

    #[inline]
    pub fn get_mut_diagnostician(&mut self) -> &mut Diagnostician {
        &mut self.diagnostician
    }
}
