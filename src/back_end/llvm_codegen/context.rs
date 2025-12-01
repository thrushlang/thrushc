use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::alloc;
use crate::back_end::llvm_codegen::anchors::PointerAnchor;
use crate::back_end::llvm_codegen::constgen;
use crate::back_end::llvm_codegen::control::LoopContext;
use crate::back_end::llvm_codegen::generation;
use crate::back_end::llvm_codegen::memory::SymbolAllocated;
use crate::back_end::llvm_codegen::memory::SymbolToAllocate;
use crate::back_end::llvm_codegen::symbols::SymbolsTable;
use crate::back_end::llvm_codegen::typegen;
use crate::back_end::llvm_codegen::types::repr::LLVMAttributes;
use crate::back_end::llvm_codegen::types::repr::LLVMFunction;

use crate::core::compiler::options::CompilerOptions;
use crate::core::diagnostic::diagnostician::Diagnostician;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::constant::ConstantMetadata;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::ast::metadata::staticvar::StaticMetadata;
use crate::front_end::types::ast::traits::AstLLVMGetType;
use crate::front_end::types::parser::repr::GlobalConstant;
use crate::front_end::types::parser::repr::GlobalStatic;
use crate::front_end::types::parser::repr::Local;
use crate::front_end::types::parser::repr::LocalConstant;
use crate::front_end::types::parser::repr::LocalStatic;
use crate::front_end::typesystem::types::Type;

use crate::logging;
use crate::logging::LoggingType;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use std::fmt::Display;
use std::path::PathBuf;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetData;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

#[derive(Debug)]
pub struct LLVMCodeGenContext<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    target_data: TargetData,

    table: SymbolsTable<'ctx>,

    loop_ctx: LoopContext<'ctx>,

    ptr_anchor: Option<PointerAnchor<'ctx>>,
    function: Option<FunctionValue<'ctx>>,

    diagnostician: Diagnostician,
    options: &'ctx CompilerOptions,
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        target_data: TargetData,
        diagnostician: Diagnostician,
        options: &'ctx CompilerOptions,
    ) -> Self {
        Self {
            module,
            context,
            builder,
            target_data,

            table: SymbolsTable::new(),
            loop_ctx: LoopContext::new(),

            ptr_anchor: None,
            function: None,

            diagnostician,
            options,
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local_constant(&mut self, constant: LocalConstant<'ctx>) {
        let name: &str = constant.0;
        let ascii_name: &str = constant.1;

        let kind: &Type = constant.2;
        let expr: &Ast = constant.3;
        let metadata: ConstantMetadata = constant.4;
        let span: Span = constant.5;

        let expr_type: &Type = expr.llvm_get_type(self);

        let compiled_value: BasicValueEnum = constgen::compile(self, expr, kind);

        let value: BasicValueEnum =
            generation::cast::try_cast_const(self, compiled_value, expr_type, kind);

        let ptr: PointerValue = alloc::memstatic::local_constant(
            self,
            ascii_name,
            typegen::generate(self.context, kind),
            value,
            metadata,
        );

        let constant: SymbolAllocated = SymbolAllocated::new_constant(
            ptr.into(),
            kind,
            value,
            metadata.get_llvm_metadata(),
            span,
        );

        if let Some(last_block) = self.table.get_mut_all_local_constants().last_mut() {
            last_block.insert(name, constant);
        } else {
            logging::print_backend_panic(
                LoggingType::BackendPanic,
                "The last frame of symbols couldn't be obtained.",
            )
        }
    }

    pub fn new_global_constant(&mut self, constant: GlobalConstant<'ctx>) {
        let name: &str = constant.0;
        let ascii_name: &str = constant.1;

        let kind: &Type = constant.2;
        let value: &Ast = constant.3;
        let attributes: LLVMAttributes = constant.4.as_llvm_attributes();
        let metadata: ConstantMetadata = constant.5;
        let span: Span = constant.6;

        let llvm_value: BasicValueEnum = constgen::compile(self, value, kind);
        let value_type: &Type = value.llvm_get_type(self);

        let value: BasicValueEnum =
            generation::cast::try_cast_const(self, llvm_value, value_type, kind);

        let ptr: PointerValue = alloc::memstatic::global_constant(
            self,
            ascii_name,
            typegen::generate(self.context, kind),
            value,
            &attributes,
            metadata,
        );

        let constant: SymbolAllocated = SymbolAllocated::new_constant(
            ptr.into(),
            kind,
            value,
            metadata.get_llvm_metadata(),
            span,
        );

        self.table
            .get_mut_all_global_constants()
            .insert(name, constant);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local_static(&mut self, staticvar: LocalStatic<'ctx>) {
        let name: &str = staticvar.0;
        let ascii_name: &str = staticvar.1;

        let kind: &Type = staticvar.2;
        let value: Option<&Ast> = staticvar.3;
        let metadata: StaticMetadata = staticvar.4;
        let span: Span = staticvar.5;

        if let Some(value) = value {
            let value_type: &Type = value.llvm_get_type(self);
            let llvm_value: BasicValueEnum = constgen::compile(self, value, kind);

            let value: BasicValueEnum =
                generation::cast::try_cast_const(self, llvm_value, value_type, kind);

            let ptr: PointerValue = alloc::memstatic::local_static(
                self,
                ascii_name,
                typegen::generate(self.context, kind),
                Some(value),
                metadata,
            );

            let staticvar: SymbolAllocated = SymbolAllocated::new_static(
                ptr.into(),
                kind,
                Some(value),
                metadata.get_llvm_metadata(),
                span,
            );

            if let Some(scope) = self.table.get_mut_all_local_statics().last_mut() {
                scope.insert(name, staticvar);
                return;
            }

            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                span,
                PathBuf::from(file!()),
                line!(),
            );
        }

        let ptr: PointerValue = alloc::memstatic::local_static(
            self,
            ascii_name,
            typegen::generate(self.context, kind),
            None,
            metadata,
        );

        let staticvar: SymbolAllocated =
            SymbolAllocated::new_static(ptr.into(), kind, None, metadata.get_llvm_metadata(), span);

        if let Some(scope) = self.table.get_mut_all_local_statics().last_mut() {
            scope.insert(name, staticvar);
            return;
        }

        abort::abort_codegen(
            self,
            "Failed to get the scope!",
            span,
            PathBuf::from(file!()),
            line!(),
        )
    }

    pub fn new_global_static(&mut self, staticvar: GlobalStatic<'ctx>) {
        let name: &str = staticvar.0;
        let ascii_name: &str = staticvar.1;

        let kind: &Type = staticvar.2;
        let value: Option<&Ast> = staticvar.3;

        let attributes: LLVMAttributes = staticvar.4.as_llvm_attributes();
        let metadata: StaticMetadata = staticvar.5;
        let span: Span = staticvar.6;

        if let Some(value) = value {
            let value_type: &Type = value.llvm_get_type(self);
            let llvm_value: BasicValueEnum = constgen::compile(self, value, kind);

            let value: BasicValueEnum =
                generation::cast::try_cast_const(self, llvm_value, value_type, kind);

            let ptr: PointerValue = alloc::memstatic::global_static(
                self,
                ascii_name,
                typegen::generate(self.context, kind),
                Some(value),
                &attributes,
                metadata,
            );

            let staticvar: SymbolAllocated = SymbolAllocated::new_static(
                ptr.into(),
                kind,
                Some(value),
                metadata.get_llvm_metadata(),
                span,
            );

            self.table
                .get_mut_all_global_statics()
                .insert(name, staticvar);

            return;
        }

        let ptr: PointerValue = alloc::memstatic::global_static(
            self,
            ascii_name,
            typegen::generate(self.context, kind),
            None,
            &attributes,
            metadata,
        );

        let staticvar: SymbolAllocated =
            SymbolAllocated::new_static(ptr.into(), kind, None, metadata.get_llvm_metadata(), span);

        self.table
            .get_mut_all_global_statics()
            .insert(name, staticvar);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn new_local(&mut self, local: Local<'ctx>) {
        let name: &str = local.0;
        let ascii_name: &str = local.1;

        let kind: &Type = local.2;
        let value: Option<&Ast> = local.3;

        let attributes: &ThrushAttributes = local.4;
        let metadata: LocalMetadata = local.5;

        let span: Span = local.6;

        let ptr: PointerValue =
            alloc::local_variable(self, ascii_name, kind, value, attributes, span);

        let local: SymbolAllocated =
            SymbolAllocated::new_local(ptr, kind, metadata.get_llvm_metadata(), span);

        if let Some(last_block) = self.table.get_mut_all_locals().last_mut() {
            last_block.insert(name, local);
        } else {
            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        }
    }

    #[inline]
    pub fn new_lli(
        &mut self,
        name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        span: Span,
    ) {
        let lli: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::LowLevelInstruction, kind, value, span);

        if let Some(last_block) = self.table.get_mut_all_locals().last_mut() {
            last_block.insert(name, lli);
        } else {
            abort::abort_codegen(
                self,
                "Failed to get the scope!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        }
    }

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

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Parameter, kind, value, span);

        self.table
            .get_mut_all_parameters()
            .insert(name, symbol_allocated);
    }

    #[inline]
    pub fn new_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.table.get_mut_all_functions().insert(name, function);
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
    pub fn set_current_fn(&mut self, new_function: FunctionValue<'ctx>) {
        self.function = Some(new_function);
    }

    #[inline]
    pub fn get_current_fn(&self) -> FunctionValue<'ctx> {
        self.function.unwrap_or_else(|| {
            self::codegen_abort("The function currently being compiled couldn't be obtained.");
        })
    }

    #[inline]
    pub fn unset_current_function(&mut self) {
        self.function = None;
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn set_pointer_anchor(&mut self, anchor: PointerAnchor<'ctx>) {
        self.ptr_anchor = Some(anchor);
    }

    #[inline]
    pub fn get_pointer_anchor(&mut self) -> Option<PointerAnchor<'ctx>> {
        self.ptr_anchor
    }

    #[inline]
    pub fn clear_pointer_anchor(&mut self) {
        self.ptr_anchor = None;
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn get_table(&self) -> &SymbolsTable<'ctx> {
        &self.table
    }

    #[inline]
    pub fn get_last_builder_block(&self) -> BasicBlock<'ctx> {
        self.builder.get_insert_block().unwrap_or_else(|| {
            self::codegen_abort("The last builder block couldn't be obtained.");
        })
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
    pub fn get_compiler_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    #[inline]
    pub fn get_mut_diagnostician(&mut self) -> &mut Diagnostician {
        &mut self.diagnostician
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    #[inline]
    pub fn get_loop_ctx(&self) -> &LoopContext<'ctx> {
        &self.loop_ctx
    }

    #[inline]
    pub fn get_mut_loop_ctx(&mut self) -> &mut LoopContext<'ctx> {
        &mut self.loop_ctx
    }
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_panic(LoggingType::BackendPanic, &format!("{}", message));
}
