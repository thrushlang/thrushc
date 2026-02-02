#![allow(clippy::too_many_arguments)]

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetData;
use inkwell::targets::TargetMachine;
use inkwell::targets::TargetTriple;
use inkwell::values::BasicValueEnum;

use inkwell::values::PointerValue;
use thrushc_diagnostician::Diagnostician;
use thrushc_options::CompilationUnit;
use thrushc_options::CompilerOptions;
use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::abort;
use crate::anchor::PointerAnchor;
use crate::brancher::LLVMLoopContext;
use crate::debug::LLVMDebugContext;
use crate::memory::SymbolAllocated;
use crate::memory::SymbolToAllocate;
use crate::optimizer::LLVMExpressionOptimization;
use crate::table::LLVMSymbolsTable;
use crate::types::LLVMCtors;
use crate::types::LLVMDBGFunction;
use crate::types::LLVMDtors;
use crate::types::LLVMFunction;

#[derive(Debug)]
pub struct LLVMCodeGenContext<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    target_data: TargetData,
    target_triple: TargetTriple,
    target_machine: &'a TargetMachine,
    dbg_context: Option<LLVMDebugContext<'a, 'ctx>>,

    table: LLVMSymbolsTable<'ctx>,
    loop_ctx: LLVMLoopContext<'ctx>,
    ctors: LLVMCtors<'ctx>,
    dtors: LLVMDtors<'ctx>,

    ptr_anchor: Option<PointerAnchor<'ctx>>,
    llvm_function: Option<LLVMFunction<'ctx>>,

    expression_optimizations: LLVMExpressionOptimization,

    diagnostician: Diagnostician,
    options: &'ctx CompilerOptions,
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        target_data: TargetData,
        target_triple: TargetTriple,
        target_machine: &'a TargetMachine,
        diagnostician: Diagnostician,
        options: &'ctx CompilerOptions,
        file: &CompilationUnit,
    ) -> Self {
        let dbg_context: Option<LLVMDebugContext> = if options
            .get_llvm_backend_options()
            .get_debug_config()
            .is_debug_mode()
        {
            Some(LLVMDebugContext::new(module, target_machine, options, file))
        } else {
            None
        };

        Self {
            module,
            context,
            builder,
            target_data,
            target_triple,
            target_machine,
            dbg_context,

            table: LLVMSymbolsTable::new(),
            loop_ctx: LLVMLoopContext::new(),

            ctors: LLVMCtors::new(),
            dtors: LLVMDtors::new(),

            ptr_anchor: None,
            llvm_function: None,

            expression_optimizations: LLVMExpressionOptimization::new(),

            diagnostician,
            options,
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn add_local_constant(&mut self, name: &'ctx str, symbol: SymbolAllocated<'ctx>) {
        if let Some(last_block) = self.table.get_mut_all_local_constants().last_mut() {
            last_block.insert(name, symbol);
        } else {
            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                symbol.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            );
        }
    }

    pub fn add_global_constant(&mut self, name: &'ctx str, symbol: SymbolAllocated<'ctx>) {
        self.table.add_global_constant(name, symbol);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn add_local_static(&mut self, name: &'ctx str, symbol: SymbolAllocated<'ctx>) {
        if let Some(scope) = self.table.get_mut_all_local_statics().last_mut() {
            scope.insert(name, symbol);
        } else {
            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                symbol.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            )
        }
    }

    pub fn add_global_static(&mut self, name: &'ctx str, static_: SymbolAllocated<'ctx>) {
        self.table.add_global_static(name, static_);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn add_local_variable(&mut self, name: &'ctx str, symbol: SymbolAllocated<'ctx>) {
        if let Some(last_block) = self.table.get_mut_all_locals().last_mut() {
            last_block.insert(name, symbol);
        } else {
            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                symbol.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            )
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn new_parameter(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        span: Span,
    ) {
        value.set_name(ascii_name);

        let parameter: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Parameter, kind, value, span);

        self.table.add_parameter(name, parameter);
    }

    #[inline]
    pub fn new_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.table.add_function(name, function);
    }
}

impl LLVMCodeGenContext<'_, '_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.table.begin_scope();
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.table.end_scope();
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn set_pointer_anchor(&mut self, anchor: PointerAnchor<'ctx>) {
        self.ptr_anchor = Some(anchor);
    }

    pub fn mark_pointer_anchor(&mut self) {
        if let Some(anchor) = &mut self.ptr_anchor {
            anchor.triggered = true;
        }
    }

    #[inline]
    pub fn clear_pointer_anchor(&mut self) {
        self.ptr_anchor = None;
    }

    #[inline]
    pub fn set_current_llvm_function(&mut self, new_function: LLVMFunction<'ctx>) {
        self.llvm_function = Some(new_function);
    }

    #[inline]
    pub fn unset_current_llvm_function(&mut self) {
        self.llvm_function = None;
    }

    #[inline]
    pub fn add_ctor(&mut self, ctor: PointerValue<'ctx>) {
        let last: Option<&(PointerValue, u32)> = self.ctors.iter().last();

        let order: u32 = if let Some(last_ctor) = last {
            last_ctor.1 + 1
        } else {
            1
        };

        self.ctors.insert((ctor, order));
    }

    #[inline]
    pub fn add_dtor(&mut self, dtor: PointerValue<'ctx>) {
        let last: Option<&(PointerValue, u32)> = self.ctors.iter().last();

        let order: u32 = if let Some(last_dtor) = last {
            last_dtor.1 + 1
        } else {
            1
        };

        self.dtors.insert((dtor, order));
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    #[inline]
    pub fn get_llvm_module(&self) -> &'a Module<'ctx> {
        self.module
    }

    #[inline]
    pub fn get_llvm_context(&self) -> &'ctx Context {
        self.context
    }

    #[inline]
    pub fn get_llvm_builder(&self) -> &'ctx Builder<'ctx> {
        self.builder
    }

    #[inline]
    pub fn get_target_data(&self) -> &TargetData {
        &self.target_data
    }

    #[inline]
    pub fn get_target_triple(&self) -> &TargetTriple {
        &self.target_triple
    }

    #[inline]
    pub fn get_target_machine(&self) -> &TargetMachine {
        self.target_machine
    }

    #[inline]
    pub fn get_debug_context(&self) -> Option<&LLVMDebugContext<'a, 'ctx>> {
        self.dbg_context.as_ref()
    }

    #[inline]
    pub fn get_compiler_options(&self) -> &CompilerOptions {
        self.options
    }

    #[inline]
    pub fn get_loop_ctx(&self) -> &LLVMLoopContext<'ctx> {
        &self.loop_ctx
    }

    #[inline]
    pub fn get_pointer_anchor(&self) -> Option<&PointerAnchor<'ctx>> {
        self.ptr_anchor.as_ref()
    }

    #[inline]
    pub fn get_llvm_ctors(&self) -> &LLVMCtors<'ctx> {
        &self.ctors
    }

    #[inline]
    pub fn get_llvm_dtors(&self) -> &LLVMDtors<'ctx> {
        &self.dtors
    }

    #[inline]
    pub fn get_table(&self) -> &LLVMSymbolsTable<'ctx> {
        &self.table
    }

    #[inline]
    pub fn get_current_llvm_function(&mut self, span: Span) -> LLVMFunction<'ctx> {
        self.llvm_function.unwrap_or_else(|| {
            abort::abort_codegen(
                self,
                "Failed to compile a function internal reference!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
    }

    #[inline]
    pub fn get_last_builder_block(&mut self, span: Span) -> BasicBlock<'ctx> {
        self.builder.get_insert_block().unwrap_or_else(|| {
            abort::abort_codegen(
                self,
                "Failed to get the last builder block!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    #[inline]
    pub fn get_mut_diagnostician(&mut self) -> &mut Diagnostician {
        &mut self.diagnostician
    }

    #[inline]
    pub fn get_mut_loop_context(&mut self) -> &mut LLVMLoopContext<'ctx> {
        &mut self.loop_ctx
    }

    #[inline]
    pub fn get_mut_debug_context(&mut self) -> Option<&mut LLVMDebugContext<'a, 'ctx>> {
        self.dbg_context.as_mut()
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn start_function_debug_data(&mut self, dbg_proto: &LLVMDBGFunction<'ctx>) {
        let mut dbg_opt: Option<LLVMDebugContext<'_, '_>> = self.dbg_context.take();

        if let Some(ref mut dbg) = dbg_opt {
            dbg.dispatch_function_debug_data(dbg_proto, self);
        }

        self.dbg_context = dbg_opt;
    }

    pub fn finish_function_debug_data(&mut self) {
        if let Some(dbg_context) = self.get_mut_debug_context() {
            dbg_context.finish_subprogram();
        }
    }

    pub fn add_dbg_block_data(&mut self, span: Span) {
        let mut dbg_opt: Option<LLVMDebugContext<'_, '_>> = self.dbg_context.take();

        if let Some(ref mut dbg) = dbg_opt {
            dbg.add_dbg_block(self, span);
        }

        self.dbg_context = dbg_opt;
    }

    pub fn mark_dbg_location(&mut self, span: Span) {
        let mut dbg_opt: Option<LLVMDebugContext<'_, '_>> = self.dbg_context.take();

        if let Some(ref mut dbg) = dbg_opt {
            dbg.add_dbg_location(self, span);
        }

        self.dbg_context = dbg_opt;
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn get_expressions_optimizations(&self) -> &LLVMExpressionOptimization {
        &self.expression_optimizations
    }

    #[inline]
    pub fn get_mut_expressions_optimizations(&mut self) -> &mut LLVMExpressionOptimization {
        &mut self.expression_optimizations
    }
}
