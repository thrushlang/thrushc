use std::rc::Rc;

use crate::middle::instruction::Instruction;
use crate::middle::statement::{FunctionParameter, FunctionPrototype, MemoryFlags};
use crate::middle::types::Type;

use super::super::compiler::attributes::LLVMAttribute;

use super::{
    attributes::{AttributeBuilder, LLVMAttributeApplicant},
    binaryop, call,
    conventions::CallConvention,
    dealloc::Deallocator,
    local, memory,
    memory::{AllocatedSymbol, MemoryFlag},
    symbols::SymbolsTable,
    typegen, unaryop, utils, valuegen,
};

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    targets::TargetData,
    types::FunctionType,
    values::{BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue},
};

pub struct Codegen<'a, 'ctx> {
    module: &'a Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    target_data: TargetData,
    instructions: &'ctx [Instruction<'ctx>],
    current: usize,
    symbols: SymbolsTable<'ctx>,
    function: Option<FunctionValue<'ctx>>,
    loop_exit_block: Option<BasicBlock<'ctx>>,
    loop_start_block: Option<BasicBlock<'ctx>>,
    deallocators_emited: bool,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn generate(
        module: &'a Module<'ctx>,
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
        target_data: TargetData,
    ) {
        Self {
            module,
            builder,
            context,
            target_data,
            instructions,
            current: 0,
            symbols: SymbolsTable::new(context, builder),
            function: None,
            loop_exit_block: None,
            loop_start_block: None,
            deallocators_emited: false,
        }
        .start();
    }

    fn start(&mut self) {
        self.declare_basics();
        self.declare();

        while !self.is_end() {
            let instruction: &Instruction = self.advance();
            self.codegen(instruction);
        }
    }

    fn codegen(&mut self, instruction: &'ctx Instruction) -> Instruction<'ctx> {
        match instruction {
            Instruction::Block { stmts, .. } => {
                self.symbols.begin_scope();

                stmts.iter().for_each(|instruction| {
                    self.codegen(instruction);
                });

                if !self.deallocators_emited {
                    let deallocator: Deallocator = Deallocator::new(
                        self.builder,
                        self.context,
                        self.symbols.get_allocated_symbols(),
                    );

                    deallocator.dealloc_all(&self.symbols);
                }

                self.deallocators_emited = false;

                self.symbols.end_scope();

                Instruction::Null
            }

            Instruction::If {
                cond,
                block,
                elfs,
                otherwise,
            } => {
                let compiled_if_cond: IntValue<'ctx> =
                    self.codegen(cond).as_llvm_value().into_int_value();

                let then_block: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "if");

                let else_if_cond: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "elseif");

                let else_if_body: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "elseif_body");

                let else_block: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "else");

                let merge_block: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "merge");

                if !elfs.is_empty() {
                    self.builder
                        .build_conditional_branch(compiled_if_cond, then_block, else_if_cond)
                        .unwrap();
                } else if otherwise.is_some() && elfs.is_empty() {
                    self.builder
                        .build_conditional_branch(compiled_if_cond, then_block, else_block)
                        .unwrap();
                } else {
                    self.builder
                        .build_conditional_branch(compiled_if_cond, then_block, merge_block)
                        .unwrap();
                }

                self.builder.position_at_end(then_block);

                self.codegen(block);

                if !block.has_return() && !block.has_break() && !block.has_continue() {
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .unwrap();
                }

                if !elfs.is_empty() {
                    self.builder.position_at_end(else_if_cond);
                } else {
                    self.builder.position_at_end(merge_block);
                }

                if !elfs.is_empty() {
                    let mut current_block: BasicBlock = else_if_body;

                    for (index, instr) in elfs.iter().enumerate() {
                        if let Instruction::Elif { cond, block } = instr {
                            let compiled_else_if_cond: IntValue =
                                self.codegen(cond).as_llvm_value().into_int_value();

                            let elif_body: BasicBlock = current_block;

                            let next_block: BasicBlock = if index + 1 < elfs.len() {
                                self.context
                                    .append_basic_block(self.function.unwrap(), "elseif_body")
                            } else if otherwise.is_some() {
                                else_block
                            } else {
                                merge_block
                            };

                            self.builder
                                .build_conditional_branch(
                                    compiled_else_if_cond,
                                    elif_body,
                                    next_block,
                                )
                                .unwrap();

                            self.builder.position_at_end(elif_body);

                            self.codegen(block);

                            if !block.has_return() && !block.has_break() && !block.has_continue() {
                                self.builder
                                    .build_unconditional_branch(merge_block)
                                    .unwrap();
                            }

                            if index + 1 < elfs.len() {
                                self.builder.position_at_end(next_block);
                                current_block = self
                                    .context
                                    .append_basic_block(self.function.unwrap(), "elseif_body");
                            }
                        }
                    }
                }

                if let Some(otherwise) = otherwise {
                    if let Instruction::Else { block } = &**otherwise {
                        self.builder.position_at_end(else_block);

                        self.codegen(block);

                        if !block.has_return() && !block.has_break() && !block.has_continue() {
                            self.builder
                                .build_unconditional_branch(merge_block)
                                .unwrap();
                        }
                    }
                }

                if !elfs.is_empty() || otherwise.is_some() {
                    self.builder.position_at_end(merge_block);
                }

                if elfs.is_empty() {
                    let _ = else_if_cond.remove_from_function();
                    let _ = else_if_body.remove_from_function();
                }

                if otherwise.is_none() {
                    let _ = else_block.remove_from_function();
                }

                Instruction::Null
            }

            Instruction::WhileLoop { cond, block } => {
                let function: FunctionValue = self.function.unwrap();

                let cond_block: BasicBlock = self.context.append_basic_block(function, "while");

                self.builder.build_unconditional_branch(cond_block).unwrap();

                self.builder.position_at_end(cond_block);

                let conditional: IntValue =
                    self.codegen(cond.as_ref()).as_llvm_value().into_int_value();

                let then_block: BasicBlock =
                    self.context.append_basic_block(function, "while_body");
                let exit_block: BasicBlock =
                    self.context.append_basic_block(function, "while_exit");

                self.loop_exit_block = Some(exit_block);

                self.builder
                    .build_conditional_branch(conditional, then_block, exit_block)
                    .unwrap();

                self.builder.position_at_end(then_block);

                self.codegen(block);

                let exit_brancher = self.builder.build_unconditional_branch(cond_block).unwrap();

                if block.has_break() {
                    exit_brancher.remove_from_basic_block();
                }

                self.builder.position_at_end(exit_block);

                Instruction::Null
            }

            Instruction::Loop { block } => {
                let function: FunctionValue = self.function.unwrap();
                let loop_start_block: BasicBlock =
                    self.context.append_basic_block(function, "loop");

                self.builder
                    .build_unconditional_branch(loop_start_block)
                    .unwrap();

                self.builder.position_at_end(loop_start_block);

                let loop_exit_block: BasicBlock = self
                    .context
                    .append_basic_block(self.function.unwrap(), "loop_exit");

                self.loop_exit_block = Some(loop_exit_block);

                self.codegen(block);

                if !block.has_return() && !block.has_break() && !block.has_continue() {
                    let _ = loop_exit_block.remove_from_function();

                    self.builder
                        .build_unconditional_branch(
                            self.function.unwrap().get_last_basic_block().unwrap(),
                        )
                        .unwrap();
                } else {
                    self.builder.position_at_end(loop_exit_block);
                }

                Instruction::Null
            }

            Instruction::ForLoop {
                variable,
                cond,
                actions,
                block,
            } => {
                let function: FunctionValue = self.function.unwrap();

                self.codegen(variable.as_ref());

                let start_block: BasicBlock = self.context.append_basic_block(function, "for");

                self.loop_start_block = Some(start_block);

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(start_block);

                let conditional: IntValue =
                    self.codegen(cond.as_ref()).as_llvm_value().into_int_value();

                let then_block: BasicBlock = self.context.append_basic_block(function, "for_body");
                let exit_block: BasicBlock = self.context.append_basic_block(function, "for_exit");

                self.builder
                    .build_conditional_branch(conditional, then_block, exit_block)
                    .unwrap();

                self.loop_exit_block = Some(exit_block);

                self.builder.position_at_end(then_block);

                if actions.is_pre_unaryop() {
                    self.codegen(block.as_ref());
                    self.codegen(actions.as_ref());
                } else {
                    self.codegen(actions.as_ref());
                    self.codegen(block.as_ref());
                }

                let exit_brancher = self
                    .builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                if block.has_break() {
                    exit_brancher.remove_from_basic_block();
                }

                self.builder.position_at_end(exit_block);

                Instruction::Null
            }

            Instruction::Break => {
                self.builder
                    .build_unconditional_branch(self.loop_exit_block.unwrap())
                    .unwrap();

                Instruction::Null
            }

            Instruction::Continue => {
                self.builder
                    .build_unconditional_branch(self.loop_start_block.unwrap())
                    .unwrap();

                Instruction::Null
            }

            Instruction::FunctionParameter {
                name,
                kind,
                position,
                ..
            } => {
                let site_allocation_flag: MemoryFlag =
                    memory::generate_site_allocation_flag(self.context, &self.target_data, kind);

                self.build_function_parameter((name, kind, *position, [site_allocation_flag]));

                Instruction::Null
            }

            Instruction::Function {
                name,
                params,
                body,
                return_type,
                attributes,
            } => {
                if let Some(body) = body {
                    self.build_function((name, return_type, params, Some(body), attributes));
                    return Instruction::Null;
                }

                Instruction::Null
            }

            Instruction::Return(kind, value) => {
                self.deallocators_emited = true;

                let deallocator: Deallocator = Deallocator::new(
                    self.builder,
                    self.context,
                    self.symbols.get_allocated_symbols(),
                );

                deallocator.dealloc(value, &self.symbols);

                valuegen::generate_expression(
                    self.module,
                    self.builder,
                    self.context,
                    instruction,
                    kind,
                    &self.symbols,
                );

                Instruction::Null
            }

            Instruction::Str(_, _, _) => Instruction::LLVMValue(valuegen::generate_expression(
                self.module,
                self.builder,
                self.context,
                instruction,
                &Type::Void,
                &self.symbols,
            )),

            Instruction::Local {
                name,
                kind,
                value,
                comptime,
                ..
            } => {
                if *comptime {
                    return Instruction::Null;
                }

                let site_allocation_flag: MemoryFlag =
                    memory::generate_site_allocation_flag(self.context, &self.target_data, kind);

                local::build(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, value, [site_allocation_flag]),
                    &mut self.symbols,
                );

                Instruction::Null
            }

            Instruction::LocalMut {
                name, kind, value, ..
            } => {
                let site_allocation_flag: MemoryFlag =
                    memory::generate_site_allocation_flag(self.context, &self.target_data, kind);

                local::build_local_mutation(
                    self.module,
                    self.builder,
                    self.context,
                    &mut self.symbols,
                    (name, kind, value, [site_allocation_flag]),
                );

                Instruction::Null
            }

            Instruction::BinaryOp {
                operator,
                left,
                right,
                kind: binaryop_type,
                ..
            } => {
                if binaryop_type.is_integer_type() {
                    return Instruction::LLVMValue(binaryop::integer::integer_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, operator, right),
                        binaryop_type,
                        &self.symbols,
                    ));
                }

                if binaryop_type.is_float_type() {
                    return Instruction::LLVMValue(binaryop::float::float_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, operator, right),
                        binaryop_type,
                        &self.symbols,
                    ));
                }

                if binaryop_type.is_bool_type() {
                    return Instruction::LLVMValue(binaryop::boolean::bool_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, operator, right),
                        binaryop_type,
                        &self.symbols,
                    ));
                }

                unimplemented!()
            }

            Instruction::UnaryOp {
                operator,
                kind,
                expression,
                ..
            } => Instruction::LLVMValue(unaryop::unary_op(
                self.builder,
                self.context,
                (operator, kind, expression),
                &self.symbols,
            )),

            Instruction::EntryPoint { body } => {
                self.function = Some(self.build_entrypoint());

                self.declare_constants();

                self.codegen(body);

                self.builder
                    .build_return(Some(&self.context.i32_type().const_int(0, false)))
                    .unwrap();

                Instruction::Null
            }

            Instruction::Call {
                name, args, kind, ..
            } => {
                call::build_call(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, args),
                    &self.symbols,
                );

                Instruction::Null
            }

            Instruction::LocalRef { kind: ref_type, .. }
            | Instruction::ConstRef { kind: ref_type, .. } => {
                Instruction::LLVMValue(valuegen::generate_expression(
                    self.module,
                    self.builder,
                    self.context,
                    instruction,
                    ref_type,
                    &self.symbols,
                ))
            }

            Instruction::Boolean(_, bool, ..) => Instruction::LLVMValue(
                self.context
                    .bool_type()
                    .const_int(*bool as u64, false)
                    .into(),
            ),

            Instruction::Address { .. } => Instruction::LLVMValue(valuegen::generate_expression(
                self.module,
                self.builder,
                self.context,
                instruction,
                &Type::Void,
                &self.symbols,
            )),

            Instruction::Write { .. } => {
                valuegen::generate_expression(
                    self.module,
                    self.builder,
                    self.context,
                    instruction,
                    &Type::Void,
                    &self.symbols,
                );

                Instruction::Null
            }

            Instruction::Null | Instruction::Const { .. } => Instruction::Null,
            e => {
                println!("{:?}", e);
                todo!()
            }
        }
    }

    fn build_entrypoint(&mut self) -> FunctionValue<'ctx> {
        let main_type: FunctionType = self.context.i32_type().fn_type(&[], false);
        let main: FunctionValue = self.module.add_function("main", main_type, None);

        let main_block: BasicBlock = self.context.append_basic_block(main, "");

        self.builder.position_at_end(main_block);

        main
    }

    fn build_function_parameter(&mut self, parameter: FunctionParameter<'ctx>) {
        let parameter_name: &str = parameter.0;
        let parameter_type: &Type = parameter.1;
        let parameter_position: u32 = parameter.2;

        let memory_flags: MemoryFlags = parameter.3;

        let llvm_parameter_value: BasicValueEnum = self
            .function
            .unwrap()
            .get_nth_param(parameter_position)
            .unwrap();

        self.symbols
            .alloc(parameter_name, parameter_type, &memory_flags);

        let symbol: AllocatedSymbol = self.symbols.get_allocated_symbol(parameter_name);

        symbol.build_store(self.builder, llvm_parameter_value);
    }

    fn declare(&mut self) {
        self.instructions.iter().for_each(|instruction| {
            if instruction.is_function() {
                self.declare_function(instruction);
            }
        });
    }

    fn declare_constants(&mut self) {
        self.instructions.iter().for_each(|instruction| {
            if let Instruction::Const {
                name,
                kind,
                value,
                attributes,
                ..
            } = instruction
            {
                let compiled_value: BasicValueEnum = valuegen::generate_expression(
                    self.module,
                    self.builder,
                    self.context,
                    value,
                    kind,
                    &self.symbols,
                );

                let constant_ptr: PointerValue = utils::build_global_constant(
                    self.module,
                    name,
                    compiled_value.get_type(),
                    compiled_value,
                    attributes,
                );

                let allocated_object: AllocatedSymbol =
                    AllocatedSymbol::alloc(constant_ptr, &[MemoryFlag::StaticAllocated], kind);

                self.symbols.insert_constant_object(name, allocated_object);
            }
        });
    }

    fn declare_function(&mut self, instruction: &'ctx Instruction) {
        let function: FunctionPrototype = instruction.as_function();

        let function_name: &str = function.0;
        let function_type: &Type = function.1;
        let function_parameters: &[Instruction] = function.2;
        let function_attributes: &[LLVMAttribute] = function.4;

        let mut call_convention: u32 = CallConvention::Standard as u32;
        let mut ignore_args: bool = false;
        let mut is_public: bool = false;
        let mut ffi: Option<&str> = None;

        function_attributes
            .iter()
            .for_each(|attribute| match attribute {
                LLVMAttribute::Public(public) => {
                    is_public = *public;
                }
                LLVMAttribute::FFI(ffi_found) => {
                    ffi = Some(ffi_found);
                }
                LLVMAttribute::Ignore => {
                    ignore_args = true;
                }
                _ => (),
            });

        let llvm_function_name: &str = if let Some(ffi_name) = ffi {
            ffi_name
        } else {
            function_name
        };

        let function_type: FunctionType = typegen::function_type(
            self.context,
            function_type,
            function_parameters,
            ignore_args,
        );

        let function: FunctionValue =
            self.module
                .add_function(llvm_function_name, function_type, None);

        let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
            self.context,
            function_attributes,
            LLVMAttributeApplicant::Function(function),
        );

        attribute_builder.add_attributes(&mut call_convention);

        if !is_public && ffi.is_none() {
            function.set_linkage(Linkage::LinkerPrivate);
        }

        self.function = Some(function);

        self.symbols.insert_function(
            function_name,
            (function, function_parameters, call_convention),
        );
    }

    fn build_function(&mut self, function: FunctionPrototype<'ctx>) {
        let function_name: &str = function.0;

        let function_type: &Type = function.1;

        let function_body: Option<&Rc<Instruction>> = function.3;

        let function_parameters: &[Instruction<'ctx>] = function.2;

        let llvm_function: FunctionValue = self.module.get_function(function_name).unwrap();

        let entry: BasicBlock = self.context.append_basic_block(llvm_function, "");

        self.builder.position_at_end(entry);

        self.codegen(function_body.unwrap());

        if function_type.is_void_type() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn declare_basics(&mut self) {
        let stderr: GlobalValue = self.module.add_global(
            self.context.ptr_type(AddressSpace::default()),
            Some(AddressSpace::default()),
            "stderr",
        );

        stderr.set_linkage(Linkage::External);

        let stdout: GlobalValue = self.module.add_global(
            self.context.ptr_type(AddressSpace::default()),
            Some(AddressSpace::default()),
            "stdout",
        );

        stdout.set_linkage(Linkage::External);
    }

    #[inline]
    fn advance(&mut self) -> &'ctx Instruction<'ctx> {
        let instruction: &Instruction = &self.instructions[self.current];
        self.current += 1;

        instruction
    }

    #[inline]
    const fn is_end(&self) -> bool {
        self.current >= self.instructions.len()
    }
}
