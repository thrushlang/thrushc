use {
    super::{
        super::{
            super::frontend::lexer::{DataTypes, TokenKind},
            instruction::Instruction,
        },
        binaryop, call, generation,
        objects::CompilerObjects,
        types::{Function, StructFields},
        unaryop, utils, variable,
    },
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        types::{FunctionType, StructType},
        values::{BasicValueEnum, FloatValue, FunctionValue, GlobalValue, IntValue, PointerValue},
        AddressSpace,
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
        }
        .start();
    }

    fn start(&mut self) {
        self.declare_basics();
        self.predefine_functions();

        while !self.is_end() {
            let instr: &Instruction<'_> = self.advance();
            self.codegen(instr);
        }
    }

    fn codegen(&mut self, instr: &'ctx Instruction<'ctx>) -> Instruction<'ctx> {
        match instr {
            Instruction::Block { stmts, .. } => {
                self.compiler_objects.push();

                stmts.iter().for_each(|instr| {
                    self.codegen(instr);
                });

                self.compiler_objects.pop();

                Instruction::Null
            }

            Instruction::Free { name, .. } => {
                let var: PointerValue<'ctx> = self.compiler_objects.find_and_get(name).unwrap();

                if self.compiler_objects.structs.contains_key(name) {
                    let struct_fields: StructFields = self.compiler_objects.get_struct_fields(name);
                    let struct_type: StructType<'_> = instr.build_struct_type(
                        self.context,
                        Some(struct_fields),
                        &mut self.compiler_objects,
                    );

                    struct_fields.iter().for_each(|field| {
                        if field.2 == DataTypes::Ptr
                            || field.2 == DataTypes::Str
                            || field.2 == DataTypes::Struct
                        {
                            let field_in_struct: PointerValue<'ctx> = self
                                .builder
                                .build_struct_gep(struct_type, var, field.3, "")
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

                self.builder.build_free(var).unwrap();

                Instruction::Null
            }

            Instruction::ForLoop {
                variable,
                cond,
                actions,
                block,
            } => {
                let function: FunctionValue<'ctx> = self.function.unwrap();

                self.codegen(variable.as_ref());

                let start_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(start_block);

                let cond: IntValue<'ctx> = self
                    .codegen(cond.as_ref())
                    .as_basic_value()
                    .into_int_value();

                let then_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");
                let exit_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");

                self.builder
                    .build_conditional_branch(cond, then_block, exit_block)
                    .unwrap();

                self.builder.position_at_end(then_block);

                self.codegen(actions.as_ref());
                self.codegen(block.as_ref());

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(exit_block);

                Instruction::Null
            }

            Instruction::Extern { .. } => Instruction::Null,

            Instruction::Param {
                name,
                kind,
                position,
                ..
            } => {
                self.build_param(name, *kind, *position);

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
                    self.build_function((name, params, Some(body), return_type, *is_public), false);
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
                &self.compiler_objects,
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
                    self.function.unwrap(),
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
                if kind.is_integer() {
                    return Instruction::BasicValueEnum(binaryop::integer_binaryop(
                        self.builder,
                        self.context,
                        (left, op, right),
                        kind,
                        &self.compiler_objects,
                    ));
                }

                if kind.is_float() {
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

            Instruction::Call { name, args, kind } => {
                call::build_call(
                    self.module,
                    self.builder,
                    self.context,
                    (name, kind, args),
                    &self.compiler_objects,
                );

                Instruction::Null
            }

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

    fn build_param(&mut self, name: &str, kind: DataTypes, position: u32) {
        let allocated_ptr: PointerValue<'ctx> = utils::build_ptr(self.context, self.builder, kind);

        let param: BasicValueEnum<'ctx> = self.function.unwrap().get_nth_param(position).unwrap();

        self.builder.build_store(allocated_ptr, param).unwrap();

        self.compiler_objects
            .insert(name.to_string(), allocated_ptr);
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
            if let DataTypes::Str = kind {
                self.builder
                    .build_return(Some(&self.compiler_objects.find_and_get(name).unwrap()))
                    .unwrap();

                return;
            }

            if kind.is_integer() {
                let num: IntValue<'_> = self
                    .builder
                    .build_load(
                        utils::datatype_integer_to_llvm_type(self.context, kind),
                        self.compiler_objects.find_and_get(name).unwrap(),
                        "",
                    )
                    .unwrap()
                    .into_int_value();

                self.builder.build_return(Some(&num)).unwrap();

                return;
            }

            if kind.is_float() {
                let num: FloatValue<'_> = self
                    .builder
                    .build_load(
                        utils::datatype_float_to_llvm_type(self.context, kind),
                        self.compiler_objects.find_and_get(name).unwrap(),
                        "",
                    )
                    .unwrap()
                    .into_float_value();

                self.builder.build_return(Some(&num)).unwrap();

                return;
            }
        }

        todo!()
    }

    fn build_external(&mut self, external_name: &str, instr: &'ctx Instruction) {
        let function: Function = instr.as_function();

        let kind: FunctionType<'_> =
            utils::datatype_to_fn_type(self.context, function.3, function.1);

        let llvm_function: FunctionValue<'_> =
            self.module
                .add_function(external_name, kind, Some(Linkage::External));

        self.compiler_objects
            .insert_function(function.0, llvm_function);
    }

    fn build_function(&mut self, function: Function<'ctx>, only_define: bool) {
        let function_name: &str = function.0;
        let function_return_type: &DataTypes = function.3;
        let function_params: &[Instruction<'_>] = function.1;
        let function_is_public: bool = function.4;
        let function_body: Option<&Box<Instruction<'ctx>>> = function.2;

        if only_define && self.module.get_function(function_name).is_none() {
            let kind: FunctionType =
                utils::datatype_to_fn_type(self.context, function_return_type, function_params);

            let function: FunctionValue<'_> = self.module.add_function(function_name, kind, None);

            if !function_is_public {
                function.set_linkage(Linkage::LinkerPrivate);
            }

            self.function = Some(function);

            self.compiler_objects
                .insert_function(function_name, function);

            return;
        }

        let function: FunctionValue<'ctx> = self.module.get_function(function_name).unwrap();

        let entry: BasicBlock = self.context.append_basic_block(function, "");

        self.builder.position_at_end(entry);

        self.codegen(function_body.unwrap());

        if *function_return_type == DataTypes::Void {
            self.builder.build_return(None).unwrap();
        }
    }

    fn predefine_functions(&mut self) {
        self.instructions.iter().for_each(|instr| {
            if instr.is_function() {
                let function: Function = instr.as_function();

                self.build_function(function, true);
            } else if instr.is_extern() {
                let external: (&str, &Instruction, TokenKind) = instr.as_extern();

                // CHECK IF TOKENKIND IS FUNCTION KIND, (REMEMBER)

                self.build_external(external.0, external.1);
            }
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
        let c: &Instruction = &self.instructions[self.current];
        self.current += 1;

        c
    }

    #[inline]
    fn is_end(&self) -> bool {
        self.current >= self.instructions.len()
    }
}
