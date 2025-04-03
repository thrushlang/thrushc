use {
    super::{
        super::{super::frontend::lexer::Type, compiler::attributes::CompilerAttribute},
        attributes::{AttributeBuilder, CompilerAttributeApplicant},
        binaryop, call,
        conventions::CallConvention,
        generation,
        instruction::Instruction,
        local,
        memory::{AllocatedObject, MemoryFlag},
        objects::CompilerObjects,
        traits::CompilerStructureFieldsExtensions,
        types::{
            CompilerStructure, CompilerStructureFields, Function, FunctionParameter, MemoryFlags,
        },
        unaryop, utils,
    },
    inkwell::{
        AddressSpace,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        targets::TargetData,
        types::{FunctionType, StructType},
        values::{BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue},
    },
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
        }
        .start();
    }

    fn start(&mut self) {
        self.declare_basics();
        self.predefine();

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

                self.compiler_objects.end_scope();

                Instruction::Null
            }

            /* Instruction::Free { name, struct_type } => {
                let struct_type: &str = struct_type;
                let variable: PointerValue = self.compiler_objects.get_allocated_object(name).ptr;

                if let Some(struct_fields) = self.compiler_objects.get_struct(struct_type) {
                    let struct_type: StructType =
                        utils::build_struct_type_from_fields(self.context, struct_fields);

                    struct_fields.iter().for_each(|field| {
                        if field.1.is_struct_type() {
                            self.build_struct_dealloc(struct_type, variable, field);
                        } else if field.1.is_ptr_type() {
                            let field_in_struct: PointerValue<'ctx> = self
                                .builder
                                .build_struct_gep(struct_type, variable, field.2, "")
                                .unwrap();

                            let loaded_field: PointerValue<'ctx> = self
                                .builder
                                .build_load(field_in_struct.get_type(), field_in_struct, "")
                                .unwrap()
                                .into_pointer_value();

                            self.builder.build_free(loaded_field).unwrap();
                        }
                    });
                }

                self.builder.build_free(variable).unwrap();

                Instruction::Null
            } */
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
                struct_type,
                ..
            } => {
                let memory_flags: MemoryFlags = self.generate_memory_flags(instruction);

                self.build_function_parameter((name, struct_type, kind, *position, memory_flags));

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

            Instruction::Return(_, kind) => {
                generation::build_expression(
                    self.module,
                    self.builder,
                    self.context,
                    instruction,
                    kind,
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
                exist_only_comptime,
                ..
            } => {
                if *exist_only_comptime {
                    return Instruction::Null;
                }

                let memory_flags: MemoryFlags = self.generate_memory_flags(value);

                local::build(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, value, memory_flags),
                    &mut self.compiler_objects,
                );

                Instruction::Null
            }

            Instruction::LocalMut { name, kind, value } => {
                let memory_flags: Vec<MemoryFlag> = self.generate_memory_flags(value);

                local::build_local_mutation(
                    self.module,
                    self.builder,
                    self.context,
                    &mut self.compiler_objects,
                    (name, kind, value, memory_flags),
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
                if kind.is_integer_type() {
                    return Instruction::LLVMValue(binaryop::integer::compile_integer_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &mut self.compiler_objects,
                    ));
                }

                if kind.is_float_type() {
                    return Instruction::LLVMValue(binaryop::float::float_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &mut self.compiler_objects,
                    ));
                }

                if kind.is_bool_type() {
                    return Instruction::LLVMValue(binaryop::boolean::bool_binaryop(
                        self.module,
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &mut self.compiler_objects,
                    ));
                }

                unimplemented!()
            }

            Instruction::UnaryOp {
                op, value, kind, ..
            } => Instruction::LLVMValue(unaryop::compile_unary_op(
                self.builder,
                self.context,
                (op, value, kind),
                &self.compiler_objects,
            )),

            Instruction::EntryPoint { body } => {
                self.function = Some(self.build_main());

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
                localref_type,
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

            e => {
                println!("{:?}", e);
                todo!()
            }
        }
    }

    fn build_main(&mut self) -> FunctionValue<'ctx> {
        let main_type: FunctionType = self.context.i32_type().fn_type(&[], false);
        let main: FunctionValue = self.module.add_function("main", main_type, None);

        let entry_point: BasicBlock = self.context.append_basic_block(main, "");

        self.builder.position_at_end(entry_point);

        main
    }

    fn build_function_parameter(&mut self, parameter: FunctionParameter<'ctx>) {
        let parameter_name: &str = parameter.0;
        let parameter_structure_type: &str = parameter.1;
        let parameter_type: Type = *parameter.2;
        let parameter_position: u32 = parameter.3;

        let memory_flags: MemoryFlags = parameter.4;

        let llvm_parameter_value: BasicValueEnum = self
            .function
            .unwrap()
            .get_nth_param(parameter_position)
            .unwrap();

        if parameter_type.is_stack_allocated() {
            let allocated_stack_pointer: PointerValue =
                utils::build_ptr(self.context, self.builder, parameter_type);

            let allocated_object: AllocatedObject =
                AllocatedObject::alloc(allocated_stack_pointer, &memory_flags);

            allocated_object.build_store(self.builder, llvm_parameter_value);

            self.compiler_objects
                .alloc_local_object(parameter_name, allocated_object);

            return;
        }

        if parameter_type.is_struct_type() {
            let structure: &CompilerStructure =
                self.compiler_objects.get_struct(parameter_structure_type);
            let structure_fields: &CompilerStructureFields = &structure.1;
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
                AllocatedObject::alloc(allocated_pointer, &memory_flags);

            allocated_object.build_store(self.builder, llvm_parameter_value);

            self.compiler_objects
                .alloc_local_object(parameter_name, allocated_object);
        }
    }

    fn predefine(&mut self) {
        self.instructions.iter().for_each(|instruction| {
            if let Instruction::Struct { name, fields_types } = instruction {
                self.compiler_objects
                    .insert_structure(name, (name, fields_types.clone()));
            }

            if instruction.is_function() {
                self.predefine_function(instruction);
            }
        });
    }

    fn predefine_function(&mut self, instruction: &'ctx Instruction) {
        let function: Function = instruction.as_function();

        let function_name: &str = function.0;
        let function_return_type: &Type = function.1;
        let function_parameters: &[Instruction] = function.2;
        let function_attributes: &[CompilerAttribute] = function.4;

        let mut call_convention: u32 = CallConvention::Standard as u32;
        let mut ignore_args: bool = false;
        let mut is_public: bool = false;
        let mut ffi: Option<&str> = None;

        function_attributes
            .iter()
            .for_each(|attribute| match attribute {
                CompilerAttribute::Public(public) => {
                    is_public = *public;
                }
                CompilerAttribute::FFI(ffi_found) => {
                    ffi = Some(ffi_found);
                }
                CompilerAttribute::Ignore => {
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
            function_return_type,
            function_parameters,
            ignore_args,
        );

        let function: FunctionValue =
            self.module
                .add_function(llvm_function_name, function_type, None);

        let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
            self.context,
            function_attributes,
            CompilerAttributeApplicant::Function(function),
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

    fn build_function(&mut self, function: Function<'ctx>) {
        let function_name: &str = function.0;
        let function_return_type: &Type = function.1;
        let function_body: Option<&Box<Instruction>> = function.3;

        let function: FunctionValue = self.module.get_function(function_name).unwrap();

        let start_block: BasicBlock = self.context.append_basic_block(function, "");

        self.builder.position_at_end(start_block);

        self.codegen(function_body.unwrap());

        if function_return_type.is_void_type() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn generate_memory_flags(&self, instruction: &'ctx Instruction) -> Vec<MemoryFlag> {
        let mut memory_flags: Vec<MemoryFlag> = Vec::with_capacity(5);

        if instruction.get_type().is_stack_allocated() {
            memory_flags.push(MemoryFlag::StackAllocated);
        }

        if let Instruction::InitStruct { name, fields, .. } = instruction {
            let mut structure_memory_size: u64 = 0;

            fields.iter().for_each(|field| {
                structure_memory_size += self
                    .target_data
                    .get_abi_size(&utils::type_to_any_type_enum(self.context, &field.2));
            });

            if fields.contain_recursive_structure_type(&self.compiler_objects, name) {
                memory_flags.push(MemoryFlag::HeapAllocated);
            } else {
                memory_flags.push(MemoryFlag::StackAllocated);
            }
        }

        memory_flags
    }

    /* fn build_struct_dealloc(
        &self,
        struct_type: StructType<'ctx>,
        variable: PointerValue,
        field: &StructField,
    ) {
        let dealloc_struct_name: String = format!("dealloc_{}_struct", field.0.to_lowercase());

        if let Some(function) = self.module.get_function(dealloc_struct_name.as_str()) {
            let gep_field_in_struct: PointerValue = self
                .builder
                .build_struct_gep(struct_type, variable, field.2, "")
                .unwrap();

            let loaded_field: PointerValue = self
                .builder
                .build_load(gep_field_in_struct.get_type(), gep_field_in_struct, "")
                .unwrap()
                .into_pointer_value();

            self.builder
                .build_call(function, &[loaded_field.into()], "")
                .unwrap();
        } else {
            let gep_field_in_struct: PointerValue = self
                .builder
                .build_struct_gep(struct_type, variable, field.2, "")
                .unwrap();

            let loaded_field: PointerValue = self
                .builder
                .build_load(gep_field_in_struct.get_type(), gep_field_in_struct, "")
                .unwrap()
                .into_pointer_value();

            self.builder.build_free(loaded_field).unwrap();
        }
    } */

    /* fn build_structure_deallocators(&self) {
        self.compiler_objects
            .structs
            .iter()
            .filter(|structure| structure.1.0.contain_heaped_fields(&self.compiler_objects))
            .for_each(|structure| {
                let dealloc_function_name: &str =
                    &format!("dealloc_{}_struct", structure.0.to_lowercase());

                let dealloc_function: FunctionValue = if let Some(dealloc_function_found) =
                    self.module.get_function(dealloc_function_name)
                {
                    dealloc_function_found
                } else {
                    self.module.add_function(
                        dealloc_function_name,
                        self.context.void_type().fn_type(
                            &[self.context.ptr_type(AddressSpace::default()).into()],
                            true,
                        ),
                        Some(Linkage::LinkerPrivate),
                    )
                };

                self.builder
                    .position_at_end(self.context.append_basic_block(dealloc_function, ""));

                let struct_pointer: PointerValue = dealloc_function
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value();

                let cmp: IntValue = self
                    .builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        dealloc_function
                            .get_nth_param(0)
                            .unwrap()
                            .into_pointer_value(),
                        self.context.ptr_type(AddressSpace::default()).const_null(),
                        "",
                    )
                    .unwrap();

                let recurse_block: BasicBlock =
                    self.context.append_basic_block(dealloc_function, "");

                let loop_exit_block: BasicBlock =
                    self.context.append_basic_block(dealloc_function, "");

                self.builder
                    .build_conditional_branch(cmp, loop_exit_block, recurse_block)
                    .unwrap();

                self.builder.position_at_end(recurse_block);

                structure
                    .1
                    .0
                    .iter()
                    .filter(|structure_field| structure_field.1.is_struct_type())
                    .for_each(|structure_field| {
                        if structure_field.0 == *structure.0 {
                            let struct_type: StructType =
                                utils::build_struct_type_from_fields(self.context, structure.1);

                            let gep_field_in_struct: PointerValue<'ctx> = self
                                .builder
                                .build_struct_gep(
                                    struct_type,
                                    struct_pointer,
                                    structure_field.2,
                                    "",
                                )
                                .unwrap();

                            let loaded_struct_from_field: PointerValue<'ctx> = self
                                .builder
                                .build_load(gep_field_in_struct.get_type(), gep_field_in_struct, "")
                                .unwrap()
                                .into_pointer_value();

                            self.builder
                                .build_call(
                                    dealloc_function,
                                    &[loaded_struct_from_field.into()],
                                    "",
                                )
                                .unwrap();

                            self.builder.build_free(struct_pointer).unwrap();
                        } else {
                            let struct_name: &str = structure_field.0;

                            let struct_fields: &Struct =
                                self.compiler_objects.structs.get(struct_name).unwrap();

                            let struct_type: StructType =
                                utils::build_struct_type_from_fields(self.context, struct_fields);

                            let gep_field_in_struct: PointerValue<'ctx> = self
                                .builder
                                .build_struct_gep(
                                    struct_type,
                                    struct_pointer,
                                    structure_field.2,
                                    "",
                                )
                                .unwrap();

                            let loaded_struct_from_field: PointerValue<'ctx> = self
                                .builder
                                .build_load(gep_field_in_struct.get_type(), gep_field_in_struct, "")
                                .unwrap()
                                .into_pointer_value();

                            let dealloc_function_name: &str =
                                &format!("dealloc_{}_struct", struct_name.to_lowercase());

                            let dealloc_function_parent: FunctionValue<'ctx> =
                                if let Some(dealloc_function_found) =
                                    self.module.get_function(dealloc_function_name)
                                {
                                    dealloc_function_found
                                } else {
                                    self.module.add_function(
                                        dealloc_function_name,
                                        self.context.void_type().fn_type(
                                            &[self
                                                .context
                                                .ptr_type(AddressSpace::default())
                                                .into()],
                                            true,
                                        ),
                                        Some(Linkage::LinkerPrivate),
                                    )
                                };

                            self.builder
                                .build_call(
                                    dealloc_function_parent,
                                    &[loaded_struct_from_field.into()],
                                    "",
                                )
                                .unwrap();
                        }
                    });

                self.builder
                    .build_unconditional_branch(loop_exit_block)
                    .unwrap();

                self.builder.position_at_end(loop_exit_block);
                self.builder.build_return(None).unwrap();
            });
    } */

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
