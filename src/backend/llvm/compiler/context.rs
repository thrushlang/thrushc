use {
    super::{
        super::super::super::logging::{self, LoggingType},
        memory::{SymbolAllocated, SymbolToAllocate},
        typegen,
    },
    crate::{
        backend::{
            llvm::compiler::{
                alloc::{self},
                anchors::PointerAnchor,
            },
            types::repr::{
                LLVMFunction, LLVMFunctions, LLVMFunctionsParameters, LLVMGlobalConstants,
                LLVMInstructions, LLVMLocalConstants,
            },
        },
        core::diagnostic::diagnostician::Diagnostician,
        frontend::types::{lexer::Type, parser::stmts::types::ThrushAttributes},
    },
    ahash::AHashMap as HashMap,
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

    global_constants: LLVMGlobalConstants<'ctx>,
    local_constants: LLVMLocalConstants<'ctx>,

    functions: LLVMFunctions<'ctx>,
    instructions: LLVMInstructions<'ctx>,
    parameters: LLVMFunctionsParameters<'ctx>,

    ptr_anchor: Option<PointerAnchor<'ctx>>,

    begin_loop_block: Option<BasicBlock<'ctx>>,
    end_loop_block: Option<BasicBlock<'ctx>>,

    function: Option<FunctionValue<'ctx>>,

    scope: usize,

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

            global_constants: HashMap::with_capacity(1000),
            local_constants: Vec::with_capacity(1000),

            functions: HashMap::with_capacity(1000),
            instructions: Vec::with_capacity(1000),
            parameters: HashMap::with_capacity(10),

            ptr_anchor: None,

            begin_loop_block: None,
            end_loop_block: None,

            function: None,

            scope: 0,

            diagnostician,
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn new_local(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        attributes: &'ctx ThrushAttributes<'ctx>,
    ) {
        let ptr: PointerValue = alloc::alloc(self, ascii_name, kind, attributes);

        let local: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Local, kind, ptr.into());

        if let Some(last_block) = self.instructions.last_mut() {
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

        if let Some(last_block) = self.instructions.last_mut() {
            last_block.insert(name, lli);
        } else {
            logging::log(
                LoggingType::BackendBug,
                "The last frame of symbols could not be obtained.",
            );
        }
    }

    pub fn new_local_constant(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
    ) {
        let ptr: PointerValue = alloc::local_constant(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
        );

        let constant: SymbolAllocated = SymbolAllocated::new_constant(ptr.into(), kind, value);

        if let Some(last_block) = self.local_constants.last_mut() {
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
    ) {
        let ptr: PointerValue = alloc::global_constant(
            self,
            ascii_name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
        );

        let constant: SymbolAllocated = SymbolAllocated::new_constant(ptr.into(), kind, value);

        self.global_constants.insert(name, constant);
    }

    pub fn new_fn_parameter(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
    ) {
        value.set_name(ascii_name);

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Parameter, kind, value);

        self.parameters.insert(name, symbol_allocated);
    }

    pub fn new_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.functions.insert(name, function);
    }
}

impl LLVMCodeGenContext<'_, '_> {
    pub fn begin_scope(&mut self) {
        self.local_constants.push(HashMap::with_capacity(256));
        self.instructions.push(HashMap::with_capacity(256));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.local_constants.pop();
        self.instructions.pop();
        self.scope -= 1;

        if self.scope == 0 {
            self.parameters.clear();
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn set_begin_loop_block(&mut self, block: BasicBlock<'ctx>) {
        self.begin_loop_block = Some(block);
    }

    pub fn get_begin_loop_block(&self) -> Option<BasicBlock<'ctx>> {
        self.begin_loop_block
    }

    pub fn set_end_loop_block(&mut self, block: BasicBlock<'ctx>) {
        self.end_loop_block = Some(block);
    }

    pub fn get_end_loop_block(&self) -> Option<BasicBlock<'ctx>> {
        self.end_loop_block
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn set_current_fn(&mut self, new_function: FunctionValue<'ctx>) {
        self.function = Some(new_function);
    }

    pub fn get_current_fn(&mut self) -> Option<FunctionValue<'ctx>> {
        self.function
    }

    pub fn clear_current_fn(&mut self) {
        self.function = None;
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn set_pointer_anchor(&mut self, anchor: PointerAnchor<'ctx>) {
        self.ptr_anchor = Some(anchor);
    }

    pub fn get_pointer_anchor(&mut self) -> Option<PointerAnchor<'ctx>> {
        self.ptr_anchor
    }

    pub fn clear_pointer_anchor(&mut self) {
        self.ptr_anchor = None;
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn get_symbol(&self, name: &str) -> SymbolAllocated<'ctx> {
        if let Some(fn_parameter) = self.parameters.get(name) {
            return *fn_parameter;
        }

        if let Some(global_constant) = self.global_constants.get(name) {
            return *global_constant;
        }

        for position in (0..self.scope).rev() {
            if let Some(local_constant) = self.local_constants[position].get(name) {
                return *local_constant;
            }
        }

        for position in (0..self.scope).rev() {
            if let Some(instruction) = self.instructions[position].get(name) {
                return *instruction;
            }
        }

        self::codegen_abort(format!(
            "Unable to get '{}' allocated object at frame pointer number #{}.",
            name, self.scope
        ));

        unreachable!()
    }

    pub fn get_function(&self, name: &str) -> LLVMFunction<'ctx> {
        if let Some(function) = self.functions.get(name) {
            return *function;
        }

        self::codegen_abort(format!(
            "Unable to get '{}' function in global frame.",
            name
        ));

        unreachable!()
    }

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
    pub fn get_diagnostician(&self) -> &Diagnostician {
        &self.diagnostician
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
