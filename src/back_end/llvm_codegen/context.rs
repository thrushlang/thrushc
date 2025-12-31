#![allow(clippy::too_many_arguments)]

use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::allocate;
use crate::back_end::llvm_codegen::constgen;
use crate::back_end::llvm_codegen::debug::LLVMDebugContext;
use crate::back_end::llvm_codegen::generation;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMAttributes;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMCtors;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMDBGFunction;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMDtors;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMFunction;
use crate::back_end::llvm_codegen::localanchor::PointerAnchor;
use crate::back_end::llvm_codegen::loopcontrol::LLVMLoopContext;
use crate::back_end::llvm_codegen::memory::SymbolAllocated;
use crate::back_end::llvm_codegen::memory::SymbolToAllocate;
use crate::back_end::llvm_codegen::symbolstable::LLVMSymbolsTable;
use crate::back_end::llvm_codegen::typegeneration;

use crate::core::compiler::options::CompilationUnit;
use crate::core::compiler::options::CompilerOptions;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;

use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

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

use std::path::PathBuf;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::targets::TargetData;
use inkwell::targets::TargetMachine;
use inkwell::targets::TargetTriple;
use inkwell::types::ArrayType;
use inkwell::types::BasicTypeEnum;
use inkwell::types::StructType;
use inkwell::values::BasicValueEnum;
use inkwell::values::GlobalValue;
use inkwell::values::PointerValue;
use inkwell::values::StructValue;

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

            diagnostician,
            options,
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn allocate_local_constant(&mut self, constant: LocalConstant<'ctx>) {
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

        let llvm_type: inkwell::types::BasicTypeEnum = typegeneration::compile_from(self, kind);

        let ptr: PointerValue = allocate::memstatic::allocate_local_constant(
            self, ascii_name, llvm_type, value, metadata,
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

    pub fn allocate_global_constant(&mut self, constant: GlobalConstant<'ctx>) {
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

        let llvm_type: BasicTypeEnum = typegeneration::compile_from(self, kind);

        let ptr: PointerValue = allocate::memstatic::allocate_global_constant(
            self, ascii_name, llvm_type, value, attributes, metadata,
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
    pub fn allocate_local_static(&mut self, staticvar: LocalStatic<'ctx>) {
        let name: &str = staticvar.0;
        let ascii_name: &str = staticvar.1;

        let kind: &Type = staticvar.2;
        let value: Option<&Ast> = staticvar.3;
        let metadata: StaticMetadata = staticvar.4;
        let span: Span = staticvar.5;

        if let Some(value) = value {
            let value_type: &Type = value.llvm_get_type(self);
            let llvm_value: BasicValueEnum = constgen::compile(self, value, kind);

            let llvm_type: BasicTypeEnum = typegeneration::compile_from(self, kind);

            let value: BasicValueEnum =
                generation::cast::try_cast_const(self, llvm_value, value_type, kind);

            let ptr: PointerValue = allocate::memstatic::allocate_local_static(
                self,
                ascii_name,
                llvm_type,
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
            } else {
                abort::abort_codegen(
                    self,
                    "Failed to get the scope!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                );
            }
        } else {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(self, kind);

            let ptr: PointerValue = allocate::memstatic::allocate_local_static(
                self, ascii_name, llvm_type, None, metadata,
            );

            let staticvar: SymbolAllocated = SymbolAllocated::new_static(
                ptr.into(),
                kind,
                None,
                metadata.get_llvm_metadata(),
                span,
            );

            if let Some(scope) = self.table.get_mut_all_local_statics().last_mut() {
                scope.insert(name, staticvar);
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
    }

    pub fn allocate_global_static(&mut self, staticvar: GlobalStatic<'ctx>) {
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

            let llvm_type: inkwell::types::BasicTypeEnum = typegeneration::compile_from(self, kind);

            let value: BasicValueEnum =
                generation::cast::try_cast_const(self, llvm_value, value_type, kind);

            let ptr: PointerValue = allocate::memstatic::allocate_global_static(
                self,
                ascii_name,
                llvm_type,
                Some(value),
                attributes,
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
        } else {
            let llvm_type: inkwell::types::BasicTypeEnum = typegeneration::compile_from(self, kind);

            let ptr: PointerValue = allocate::memstatic::allocate_global_static(
                self, ascii_name, llvm_type, None, attributes, metadata,
            );

            let staticvar: SymbolAllocated = SymbolAllocated::new_static(
                ptr.into(),
                kind,
                None,
                metadata.get_llvm_metadata(),
                span,
            );

            self.table
                .get_mut_all_global_statics()
                .insert(name, staticvar);
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    #[inline]
    pub fn allocate_local(&mut self, local: Local<'ctx>) {
        let name: &str = local.0;
        let ascii_name: &str = local.1;

        let kind: &Type = local.2;
        let value: Option<&Ast> = local.3;

        let attributes: &ThrushAttributes = local.4;
        let metadata: LocalMetadata = local.5;

        let span: Span = local.6;

        let ptr: PointerValue =
            allocate::memstack::local_variable(self, ascii_name, kind, value, attributes, span);

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
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
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
    pub fn set_pointer_anchor(&mut self, anchor: PointerAnchor<'ctx>) {
        self.ptr_anchor = Some(anchor);
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
    pub fn get_mut_debug_context(&mut self) -> Option<&mut LLVMDebugContext<'a, 'ctx>> {
        self.dbg_context.as_mut()
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
    pub fn get_pointer_anchor(&mut self) -> Option<PointerAnchor<'ctx>> {
        self.ptr_anchor
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
                PathBuf::from(file!()),
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
                PathBuf::from(file!()),
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
    pub fn get_mut_loop_ctx(&mut self) -> &mut LLVMLoopContext<'ctx> {
        &mut self.loop_ctx
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

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn generate_constructors(&mut self, span: Span) {
        if self.ctors.is_empty() {
            return;
        }

        let llvm_context: &Context = self.get_llvm_context();
        let llvm_module: &Module = self.get_llvm_module();

        let ctor_type: StructType = llvm_context.struct_type(
            &[
                llvm_context.i32_type().into(),
                llvm_context.ptr_type(AddressSpace::default()).into(),
                llvm_context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        );

        let mut llvm_ctors: Vec<StructValue> = Vec::with_capacity(self.ctors.len());
        let mut last_counter: u32 = 0;

        for (ctor, counter) in self.get_llvm_ctors().iter() {
            if *counter > last_counter {
                let ctor_value: StructValue = ctor_type.const_named_struct(&[
                    llvm_context
                        .i32_type()
                        .const_int((*counter).into(), false)
                        .into(),
                    (*ctor).into(),
                    llvm_context
                        .ptr_type(AddressSpace::default())
                        .const_null()
                        .into(),
                ]);

                llvm_ctors.push(ctor_value);
                last_counter = *counter;
            }
        }

        let actual_size: u32 = u32::try_from(llvm_ctors.len()).unwrap_or_else(|_| {
            abort::abort_codegen(
                self,
                "Failed to parse the size for the ctors!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        });

        let llvm_ctors_type: ArrayType = ctor_type.array_type(actual_size);
        let global: GlobalValue =
            llvm_module.add_global(llvm_ctors_type, None, "llvm.global_ctors");

        global.set_linkage(Linkage::Appending);
        global.set_initializer(&ctor_type.const_array(&llvm_ctors));
    }

    pub fn generate_destructors(&mut self, span: Span) {
        if self.dtors.is_empty() {
            return;
        }

        let llvm_context: &Context = self.get_llvm_context();
        let llvm_module: &Module = self.get_llvm_module();

        let dtor_type: StructType = llvm_context.struct_type(
            &[
                llvm_context.i32_type().into(),
                llvm_context.ptr_type(AddressSpace::default()).into(),
                llvm_context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        );

        let mut llvm_dtors: Vec<StructValue> = Vec::with_capacity(self.dtors.len());
        let mut last_counter: u32 = 0;

        for (ctor, counter) in self.get_llvm_dtors().iter() {
            if *counter > last_counter {
                let dtor_value: StructValue = dtor_type.const_named_struct(&[
                    llvm_context
                        .i32_type()
                        .const_int((*counter).into(), false)
                        .into(),
                    (*ctor).into(),
                    llvm_context
                        .ptr_type(AddressSpace::default())
                        .const_null()
                        .into(),
                ]);

                llvm_dtors.push(dtor_value);
                last_counter = *counter;
            }
        }

        let actual_size: u32 = u32::try_from(llvm_dtors.len()).unwrap_or_else(|_| {
            abort::abort_codegen(
                self,
                "Failed to parse the size for the dtors!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        });

        let llvm_dtors_type: ArrayType = dtor_type.array_type(actual_size);
        let global: GlobalValue =
            llvm_module.add_global(llvm_dtors_type, None, "llvm.global_dtors");

        global.set_linkage(Linkage::Appending);
        global.set_initializer(&dtor_type.const_array(&llvm_dtors));
    }
}
