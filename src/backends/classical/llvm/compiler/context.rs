use crate::logging::{self, LoggingType};

use crate::backends::classical::types::repr::LLVMFunction;

use crate::backends::classical::llvm::compiler::memory::SymbolAllocated;
use crate::backends::classical::llvm::compiler::memory::SymbolToAllocate;
use crate::backends::classical::llvm::compiler::typegen;

use crate::backends::classical::llvm::compiler::alloc;
use crate::backends::classical::llvm::compiler::anchors::PointerAnchor;
use crate::backends::classical::llvm::compiler::control::LoopContext;
use crate::backends::classical::llvm::compiler::symbols::SymbolsTable;

use crate::core::diagnostic::diagnostician::Diagnostician;

use crate::frontends::classical::types::ast::metadata::constant::ConstantMetadata;
use crate::frontends::classical::types::ast::metadata::local::LocalMetadata;
use crate::frontends::classical::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

use {
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        targets::TargetData,
        values::{BasicValueEnum, FunctionValue, PointerValue},
    },
    std::fmt::Display,
};

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
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        target_data: TargetData,
        diagnostician: Diagnostician,
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
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local_constant(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,

        metadata: ConstantMetadata,
    ) {
        let ptr: PointerValue = alloc::memstatic::local_constant(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
            metadata,
        );

        let constant: SymbolAllocated =
            SymbolAllocated::new_constant(ptr.into(), kind, value, metadata.get_llvm_metadata());

        if let Some(last_block) = self.table.get_mut_local_constants().last_mut() {
            last_block.insert(name, constant);
        } else {
            logging::log(
                LoggingType::BackendBug,
                "The last frame of symbols could not be obtained.",
            )
        }
    }

    pub fn new_global_constant(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        attributes: &'ctx ThrushAttributes<'ctx>,

        metadata: ConstantMetadata,
    ) {
        let ptr: PointerValue = alloc::memstatic::global_constant(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
            metadata,
        );

        let constant: SymbolAllocated =
            SymbolAllocated::new_constant(ptr.into(), kind, value, metadata.get_llvm_metadata());

        self.table.get_mut_global_constants().insert(name, constant);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local_static(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,

        metadata: StaticMetadata,
    ) {
        let ptr: PointerValue = alloc::memstatic::local_static(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
            metadata,
        );

        let constant: SymbolAllocated =
            SymbolAllocated::new_static(ptr.into(), kind, value, metadata.get_llvm_metadata());

        if let Some(last_block) = self.table.get_mut_local_statics().last_mut() {
            last_block.insert(name, constant);
        } else {
            logging::log(
                LoggingType::BackendBug,
                "The last frame of symbols could not be obtained.",
            )
        }
    }

    pub fn new_global_static(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        attributes: &'ctx ThrushAttributes<'ctx>,

        metadata: StaticMetadata,
    ) {
        let ptr: PointerValue = alloc::memstatic::global_static(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
            metadata,
        );

        let constant: SymbolAllocated =
            SymbolAllocated::new_static(ptr.into(), kind, value, metadata.get_llvm_metadata());

        self.table.get_mut_global_statics().insert(name, constant);
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        attributes: &'ctx ThrushAttributes<'ctx>,

        metadata: LocalMetadata,
    ) {
        let ptr: PointerValue = alloc::alloc(self, ascii_name, kind, attributes);

        let local: SymbolAllocated =
            SymbolAllocated::new_local(ptr, kind, metadata.get_llvm_metadata());

        if let Some(last_block) = self.table.get_mut_locals().last_mut() {
            last_block.insert(name, local);
        } else {
            logging::log(
                LoggingType::BackendBug,
                "The last frame of symbols could not be obtained.",
            );
        }
    }

    pub fn new_lli(&mut self, name: &'ctx str, kind: &'ctx Type, value: BasicValueEnum<'ctx>) {
        let lli: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::LowLevelInstruction, kind, value);

        if let Some(last_block) = self.table.get_mut_locals().last_mut() {
            last_block.insert(name, lli);
        } else {
            logging::log(
                LoggingType::BackendBug,
                "The last frame of symbols could not be obtained.",
            );
        }
    }

    pub fn new_parameter(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
    ) {
        value.set_name(ascii_name);

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Parameter, kind, value);

        self.table
            .get_mut_parameters()
            .insert(name, symbol_allocated);
    }

    pub fn new_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.table.get_mut_functions().insert(name, function);
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
            self::codegen_abort("The function currently being compiled could not be obtained.");
            unreachable!()
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
            self::codegen_abort("The last builder block could not be obtained.");
            unreachable!()
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
    pub fn get_loop_ctx(&self) -> &LoopContext {
        &self.loop_ctx
    }

    #[inline]
    pub fn get_mut_loop_ctx(&mut self) -> &mut LoopContext<'ctx> {
        &mut self.loop_ctx
    }

    #[inline]
    pub fn get_diagnostician(&self) -> &Diagnostician {
        &self.diagnostician
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
