use {
    super::{
        super::{
            super::frontend::lexer::{DataTypes, TokenKind},
            instruction::Instruction,
        },
        binaryop, call, generation,
        objects::CompilerObjects,
        traits::StructureBasics,
        types::{Function, Struct, StructField},
        unaryop, utils, variable,
    },
    core::str,
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        types::{FunctionType, StructType},
        values::{BasicValueEnum, FloatValue, FunctionValue, GlobalValue, IntValue, PointerValue},
        AddressSpace, IntPredicate,
    },
};

pub struct Codegen<'a, 'ctx> {
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    context: &'ctx Context,
    instructions: &'ctx [Instruction<'ctx>],
    current: usize,
    compiler_objects: CompilerObjects<'ctx>,
    function: Option<FunctionValue<'ctx>>,
    loop_exit_block: Option<BasicBlock<'ctx>>,
    loop_start_block: Option<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn gen(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
    ) {
        Self {
            module,
            builder,
            context,
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

        // Build recursive deallocators for structures.
        self.build_structure_deallocators();

        while !self.is_end() {
            let instruction: &Instruction = self.advance();
            self.codegen(instruction);
        }
    }

    fn codegen(&mut self, instr: &'ctx Instruction) -> Instruction<'ctx> {
        match instr {
            Instruction::Block { stmts, .. } => {
                self.compiler_objects.begin_scope();

                stmts.iter().for_each(|instr| {
                    self.codegen(instr);
                });

                self.compiler_objects.end_scope();

                Instruction::Null
            }

            Instruction::Free { name, struct_type } => {
                let struct_type: &str = struct_type;
                let variable: PointerValue<'ctx> = self.compiler_objects.get_local(name).unwrap();

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
            }

            Instruction::If {
                cond,
                block,
                elfs,
                otherwise,
            } => {
                let compiled_if_cond: IntValue<'ctx> =
                    self.codegen(cond).as_basic_value().into_int_value();

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

                if elfs.is_some() {
                    self.builder
                        .build_conditional_branch(compiled_if_cond, then_block, else_if_cond)
                        .unwrap();
                } else if otherwise.is_some() && elfs.is_none() {
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

                if elfs.is_some() {
                    self.builder.position_at_end(else_if_cond);
                } else {
                    self.builder.position_at_end(merge_block);
                }

                if let Some(chained_elifs) = elfs {
                    let mut current_block: BasicBlock = else_if_body;

                    for (index, instr) in chained_elifs.iter().enumerate() {
                        if let Instruction::Elif { cond, block } = instr {
                            let compiled_else_if_cond: IntValue =
                                self.codegen(cond).as_basic_value().into_int_value();

                            let elif_body: BasicBlock = current_block;

                            let next_block: BasicBlock = if index + 1 < chained_elifs.len() {
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

                            if index + 1 < chained_elifs.len() {
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

                if elfs.is_some() || otherwise.is_some() {
                    self.builder.position_at_end(merge_block);
                }

                if elfs.is_none() {
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

                let conditional: IntValue = self
                    .codegen(cond.as_ref())
                    .as_basic_value()
                    .into_int_value();

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

                let start_block: BasicBlock =
                    self.context.append_basic_block(function, "for_start");

                self.loop_start_block = Some(start_block);

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(start_block);

                let conditional: IntValue = self
                    .codegen(cond.as_ref())
                    .as_basic_value()
                    .into_int_value();

                let then_block: BasicBlock = self.context.append_basic_block(function, "for_body");
                let exit_block: BasicBlock = self.context.append_basic_block(function, "for_exit");

                self.builder
                    .build_conditional_branch(conditional, then_block, exit_block)
                    .unwrap();

                self.loop_exit_block = Some(exit_block);

                self.builder.position_at_end(then_block);

                self.codegen(actions.as_ref());

                self.codegen(block.as_ref());

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

            Instruction::Extern { .. } => Instruction::Null,

            Instruction::FunctionParameter {
                name,
                kind,
                position,
                ..
            } => {
                self.build_function_parameter(name, *kind, *position);
                Instruction::Null
            }

            Instruction::Function {
                name,
                params,
                body,
                return_type,
                is_public,
            } => {
                if let Some(body) = body {
                    self.build_function((name, params, Some(body), return_type, is_public), false);
                    return Instruction::Null;
                }

                Instruction::Null
            }

            Instruction::Return(instr, kind) => {
                self.build_return(instr, kind);
                Instruction::Null
            }

            Instruction::Str(_) => Instruction::BasicValueEnum(generation::build_basic_value_enum(
                self.module,
                self.builder,
                self.context,
                instr,
                None,
                &mut self.compiler_objects,
            )),

            Instruction::Var {
                name,
                kind,
                value,
                exist_only_comptime,
                ..
            } => {
                if *exist_only_comptime {
                    return Instruction::Null;
                }

                variable::compile(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, value),
                    &mut self.compiler_objects,
                );

                Instruction::Null
            }

            Instruction::MutVar { name, kind, value } => {
                variable::compile_mut(
                    self.module,
                    self.builder,
                    self.context,
                    &mut self.compiler_objects,
                    (name, kind, value),
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
                    return Instruction::BasicValueEnum(binaryop::integer_binaryop(
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &self.compiler_objects,
                    ));
                }

                if kind.is_float_type() {
                    return Instruction::BasicValueEnum(binaryop::float_binaryop(
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &self.compiler_objects,
                    ));
                }

                if *kind == DataTypes::Bool {
                    return Instruction::BasicValueEnum(binaryop::bool_binaryop(
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &self.compiler_objects,
                    ));
                }

                unimplemented!()
            }

            Instruction::UnaryOp { op, value, kind } => {
                Instruction::BasicValueEnum(unaryop::compile_unary_op(
                    self.builder,
                    self.context,
                    (op, value, kind),
                    &self.compiler_objects,
                ))
            }

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

            Instruction::Boolean(bool) => Instruction::BasicValueEnum(
                self.context
                    .bool_type()
                    .const_int(*bool as u64, false)
                    .into(),
            ),

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

    fn build_function_parameter(&mut self, name: &'ctx str, kind: DataTypes, position: u32) {
        let allocated_ptr: PointerValue<'ctx> = if !kind.is_ptr_type() {
            utils::build_ptr(self.context, self.builder, kind)
        } else {
            self.function
                .unwrap()
                .get_nth_param(position)
                .unwrap()
                .into_pointer_value()
        };

        if !kind.is_ptr_type() {
            let param: BasicValueEnum<'ctx> =
                self.function.unwrap().get_nth_param(position).unwrap();

            self.builder.build_store(allocated_ptr, param).unwrap();

            self.compiler_objects.insert(name, allocated_ptr);

            return;
        }

        self.compiler_objects.insert(name, allocated_ptr);
    }

    fn build_return(&mut self, instr: &'ctx Instruction, kind: &DataTypes) {
        if *kind == DataTypes::Void {
            self.builder.build_return(None).unwrap();
            return;
        }

        if let Instruction::Integer(_, num, is_signed) = instr {
            self.builder
                .build_return(Some(&utils::build_const_integer(
                    self.context,
                    kind,
                    *num as u64,
                    *is_signed,
                )))
                .unwrap();

            return;
        }

        if let Instruction::Str(_) = instr {
            self.builder
                .build_return(Some(self.codegen(instr).as_basic_value()))
                .unwrap();

            return;
        }

        if let Instruction::Char(byte) = instr {
            self.builder
                .build_return(Some(&self.context.i8_type().const_int(*byte as u64, false)))
                .unwrap();

            return;
        }

        if let Instruction::Boolean(bool) = instr {
            self.builder
                .build_return(Some(
                    &self.context.bool_type().const_int(*bool as u64, false),
                ))
                .unwrap();

            return;
        }

        if let Instruction::RefVar { name, .. } = instr {
            let variable: PointerValue<'ctx> = self.compiler_objects.get_local(name).unwrap();

            if kind.is_integer_type() || *kind == DataTypes::Bool {
                let num: IntValue = self
                    .builder
                    .build_load(
                        utils::datatype_integer_to_llvm_type(self.context, kind),
                        variable,
                        "",
                    )
                    .unwrap()
                    .into_int_value();

                self.builder.build_return(Some(&num)).unwrap();

                return;
            }

            if kind.is_float_type() {
                let num: FloatValue = self
                    .builder
                    .build_load(
                        utils::datatype_float_to_llvm_type(self.context, kind),
                        variable,
                        "",
                    )
                    .unwrap()
                    .into_float_value();

                self.builder.build_return(Some(&num)).unwrap();

                return;
            }

            if kind.is_ptr_type() {
                self.builder.build_return(Some(&variable)).unwrap();
                return;
            }
        }

        unreachable!()
    }

    fn build_struct_dealloc(
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
    }

    fn build_external(&mut self, external_name: &str, instr: &'ctx Instruction) {
        let function: Function = instr.as_function();

        let kind: FunctionType = utils::datatype_to_fn_type(self.context, function.3, function.1);

        let external_function: FunctionValue =
            self.module
                .add_function(external_name, kind, Some(Linkage::External));

        self.compiler_objects
            .insert_function(function.0, (external_function, function.1));
    }

    fn build_function(&mut self, function: Function<'ctx>, only_define: bool) {
        let function_name: &str = function.0;
        let function_return_type: &DataTypes = function.3;
        let function_params: &[Instruction] = function.1;
        let function_is_public: &bool = function.4;
        let function_body: Option<&Box<Instruction>> = function.2;

        if only_define && self.module.get_function(function_name).is_none() {
            let kind: FunctionType =
                utils::datatype_to_fn_type(self.context, function_return_type, function_params);

            let function: FunctionValue = self.module.add_function(function_name, kind, None);

            if !function_is_public {
                function.set_linkage(Linkage::LinkerPrivate);
            }

            self.function = Some(function);

            self.compiler_objects
                .insert_function(function_name, (function, function_params));

            return;
        }

        let function: FunctionValue = self.module.get_function(function_name).unwrap();

        let entry: BasicBlock = self.context.append_basic_block(function, "");

        self.builder.position_at_end(entry);

        self.codegen(function_body.unwrap());

        if function_return_type.is_void_type() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn predefine(&mut self) {
        self.instructions.iter().for_each(|instr| {
            if instr.is_function() {
                let function: Function = instr.as_function();

                self.build_function(function, true);
            } else if instr.is_extern() {
                let external: (&str, &Instruction, &TokenKind) = instr.as_extern();

                // CHECK IF TOKENKIND IS FUNCTION KIND, (REMEMBER)

                self.build_external(external.0, external.1);
            } else if let Instruction::Struct { name, fields_types } = instr {
                self.compiler_objects.insert_struct(name, fields_types);
            }
        });
    }

    fn build_structure_deallocators(&self) {
        self.compiler_objects
            .structs
            .iter()
            .filter(|structure| structure.1.contain_heaped_fields(&self.compiler_objects))
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
        let instr: &Instruction = &self.instructions[self.current];
        self.current += 1;

        instr
    }

    #[inline]
    const fn is_end(&self) -> bool {
        self.current >= self.instructions.len()
    }
}
