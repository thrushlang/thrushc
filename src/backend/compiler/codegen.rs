use super::super::super::frontend::lexer::Type;

use super::super::compiler::attributes::LLVMAttribute;

use super::types::FunctionPrototype;

use super::{
    attributes::{AttributeBuilder, LLVMAttributeApplicant},
    binaryop, call,
    conventions::CallConvention,
    dealloc::Deallocator,
    generation,
    instruction::Instruction,
    local,
    memory::{AllocatedObject, MemoryFlag},
    objects::CompilerObjects,
    traits::CompilerStructureFieldsExtensions,
    types::{FunctionParameter, MemoryFlags, Structure, StructureFields},
    unaryop, utils,
};

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    targets::TargetData,
    types::{FunctionType, StructType},
    values::{BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue},
};

pub struct Codegen<'a, 'ctx> {
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    context: &'ctx Context,
    target_data: TargetData,
    instructions: &'ctx [Instruction<'ctx>],
    current: usize,
    compiler_objects: CompilerObjects<'ctx>,
    function: Option<FunctionValue<'ctx>>,
    loop_exit_block: Option<BasicBlock<'ctx>>,
    loop_start_block: Option<BasicBlock<'ctx>>,
    deallocators_emited: bool,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn generate(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
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
            compiler_objects: CompilerObjects::new(),
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
                self.compiler_objects.begin_scope();

                stmts.iter().for_each(|instruction| {
                    self.codegen(instruction);
                });

                if !self.deallocators_emited {
                    let deallocator: Deallocator = Deallocator::new(
                        self.builder,
                        self.context,
                        self.compiler_objects.get_allocated_objects(),
                    );

                    deallocator.dealloc_all(&self.compiler_objects);
                }

                self.deallocators_emited = false;

                self.compiler_objects.end_scope();

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
                    self.generate_site_allocation_flag(instruction);

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

            Instruction::Return(return_instruction, kind) => {
                self.deallocators_emited = true;

                let basic_type: &Type = kind.get_basic_type();

                let deallocator: Deallocator = Deallocator::new(
                    self.builder,
                    self.context,
                    self.compiler_objects.get_allocated_objects(),
                );

                deallocator.dealloc(return_instruction, &self.compiler_objects);

                generation::build_expression(
                    self.module,
                    self.builder,
                    self.context,
                    instruction,
                    basic_type,
                    &mut self.compiler_objects,
                );

                Instruction::Null
            }

            Instruction::Str(_) => Instruction::LLVMValue(generation::build_expression(
                self.module,
                self.builder,
                self.context,
                instruction,
                &Type::Void,
                &mut self.compiler_objects,
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

                let site_allocation_flag: MemoryFlag = self.generate_site_allocation_flag(value);

                local::build(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, value, [site_allocation_flag]),
                    &mut self.compiler_objects,
                );

                Instruction::Null
            }

            Instruction::LocalMut { name, kind, value } => {
                let site_allocation_flag: MemoryFlag = self.generate_site_allocation_flag(value);

                local::build_local_mutation(
                    self.module,
                    self.builder,
                    self.context,
                    &mut self.compiler_objects,
                    (name, kind, value, [site_allocation_flag]),
                );

                Instruction::Null
            }

            Instruction::BinaryOp {
                op,
                left,
                right,
                kind,
                ..
            } => {
                let binaryop_type: &Type = kind.get_basic_type();

                if binaryop_type.is_integer_type() {
                    return Instruction::LLVMValue(binaryop::integer::compile_integer_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        binaryop_type,
                        &mut self.compiler_objects,
                    ));
                }

                if binaryop_type.is_float_type() {
                    return Instruction::LLVMValue(binaryop::float::float_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        binaryop_type,
                        &mut self.compiler_objects,
                    ));
                }

                if binaryop_type.is_bool_type() {
                    return Instruction::LLVMValue(binaryop::boolean::bool_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        binaryop_type,
                        &mut self.compiler_objects,
                    ));
                }

                unimplemented!()
            }

            Instruction::UnaryOp {
                op,
                expression,
                kind,
                ..
            } => Instruction::LLVMValue(unaryop::compile_unary_op(
                self.builder,
                self.context,
                (op, expression, kind),
                &self.compiler_objects,
            )),

            Instruction::EntryPoint { body } => {
                self.function = Some(self.build_entrypoint());

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
                    &mut self.compiler_objects,
                );

                Instruction::Null
            }

            Instruction::LocalRef {
                kind: localref_type,
                ..
            } => Instruction::LLVMValue(generation::build_expression(
                self.module,
                self.builder,
                self.context,
                instruction,
                localref_type.get_basic_type(),
                &mut self.compiler_objects,
            )),

            Instruction::Boolean(bool) => Instruction::LLVMValue(
                self.context
                    .bool_type()
                    .const_int(*bool as u64, false)
                    .into(),
            ),

            Instruction::GEP { .. } => Instruction::LLVMValue(generation::build_expression(
                self.module,
                self.builder,
                self.context,
                instruction,
                &Type::U64,
                &mut self.compiler_objects,
            )),

            Instruction::Struct { .. } => Instruction::Null,
            Instruction::Null => Instruction::Null,
            Instruction::Comptime => Instruction::Null,

            e => {
                println!("{:?}", e);
                todo!()
            }
        }
    }

    fn build_entrypoint(&mut self) -> FunctionValue<'ctx> {
        let main_type: FunctionType = self.context.i32_type().fn_type(&[], false);
        let main: FunctionValue = self.module.add_function("main", main_type, None);

        let entry_point: BasicBlock = self.context.append_basic_block(main, "");

        self.builder.position_at_end(entry_point);

        main
    }

    fn build_function_parameter(&mut self, parameter: FunctionParameter<'ctx>) {
        let parameter_name: &str = parameter.0;
        let parameter_type: &Instruction = parameter.1;
        let parameter_basic_type: &Type = parameter.1.get_basic_type();
        let parameter_position: u32 = parameter.2;

        let memory_flags: MemoryFlags = parameter.3;

        let llvm_parameter_value: BasicValueEnum = self
            .function
            .unwrap()
            .get_nth_param(parameter_position)
            .unwrap();

        if parameter_basic_type.is_stack_allocated() {
            let allocated_stack_pointer: PointerValue =
                utils::build_ptr(self.context, self.builder, parameter_basic_type);

            let allocated_object: AllocatedObject =
                AllocatedObject::alloc(allocated_stack_pointer, &memory_flags, parameter_type);

            allocated_object.build_store(self.builder, llvm_parameter_value);

            self.compiler_objects
                .alloc_local_object(parameter_name, allocated_object);

            return;
        }

        if parameter_basic_type.is_struct_type() {
            let parameter_structure_type: &str = parameter.1.get_structure_type();

            let structure: &Structure = self.compiler_objects.get_struct(parameter_structure_type);
            let structure_fields: &StructureFields = &structure.1;

            let llvm_structure_type: StructType =
                utils::build_struct_type_from_fields(self.context, structure_fields);

            let allocated_pointer: PointerValue = if structure_fields
                .contain_recursive_structure_type(&self.compiler_objects, parameter_structure_type)
            {
                self.builder.build_malloc(llvm_structure_type, "").unwrap()
            } else {
                self.builder.build_alloca(llvm_structure_type, "").unwrap()
            };

            let allocated_object: AllocatedObject =
                AllocatedObject::alloc(allocated_pointer, &memory_flags, parameter_type);

            allocated_object.build_store(self.builder, llvm_parameter_value);

            self.compiler_objects
                .alloc_local_object(parameter_name, allocated_object);
        }
    }

    fn declare(&mut self) {
        self.instructions.iter().for_each(|instruction| {
            if let Instruction::Struct { name, fields_types } = instruction {
                self.compiler_objects
                    .insert_structure(name, (name, fields_types.clone()));
            }

            if instruction.is_function() {
                self.declare_function(instruction);
            }
        });
    }

    fn declare_function(&mut self, instruction: &'ctx Instruction) {
        let function: FunctionPrototype = instruction.as_function();

        let function_name: &str = function.0;
        let function_type: &Instruction = function.1;
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

        let function_type: FunctionType = utils::type_to_function_type(
            self.context,
            &self.compiler_objects,
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

        self.compiler_objects.insert_function(
            function_name,
            (function, function_parameters, call_convention),
        );
    }

    fn build_function(&mut self, function: FunctionPrototype<'ctx>) {
        let function_name: &str = function.0;

        let function_type: &Instruction = function.1;
        let function_basic_type: &Type = function_type.get_basic_type();

        let function_body: Option<&Box<Instruction>> = function.3;

        let llvm_function: FunctionValue = self.module.get_function(function_name).unwrap();

        let entry: BasicBlock = self.context.append_basic_block(llvm_function, "");

        self.builder.position_at_end(entry);

        self.codegen(function_body.unwrap());

        if function_basic_type.is_void_type() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn generate_site_allocation_flag(&self, instruction: &'ctx Instruction) -> MemoryFlag {
        const MAX_STACK_SIZE_OF_STRUCTURE: u64 = 128;

        let mut alloc_site_memory_flag: MemoryFlag = MemoryFlag::StackAllocated;

        if let Instruction::InitStruct { name, .. } = instruction {
            let mut structure_memory_size: u64 = 0;

            let structure: &Structure = self.compiler_objects.get_struct(name);
            let structure_fields: &StructureFields = &structure.1;

            structure_fields.iter().for_each(|field| {
                let field_basic_type: &Type = field.1.get_basic_type();

                structure_memory_size += self.target_data.get_abi_size(
                    &utils::type_to_any_type_enum(self.context, field_basic_type),
                );
            });

            if structure_fields.contain_recursive_structure_type(&self.compiler_objects, name)
                || structure_memory_size >= MAX_STACK_SIZE_OF_STRUCTURE
            {
                alloc_site_memory_flag = MemoryFlag::HeapAllocated;
            } else {
                alloc_site_memory_flag = MemoryFlag::StackAllocated;
            }
        }

        if let Instruction::FunctionParameter { kind, .. } = instruction {
            if kind.get_basic_type().is_struct_type() {
                let structure_type: &str = kind.get_structure_type();

                let mut structure_memory_size: u64 = 0;

                let structure: &Structure = self.compiler_objects.get_struct(structure_type);
                let structure_fields: &StructureFields = &structure.1;

                structure_fields.iter().for_each(|field| {
                    let field_basic_type: &Type = field.1.get_basic_type();

                    structure_memory_size += self.target_data.get_abi_size(
                        &utils::type_to_any_type_enum(self.context, field_basic_type),
                    );
                });

                if structure_fields
                    .contain_recursive_structure_type(&self.compiler_objects, structure_type)
                    || structure_memory_size >= MAX_STACK_SIZE_OF_STRUCTURE
                {
                    alloc_site_memory_flag = MemoryFlag::HeapAllocated;
                } else {
                    alloc_site_memory_flag = MemoryFlag::StackAllocated;
                }
            }
        }

        alloc_site_memory_flag
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
