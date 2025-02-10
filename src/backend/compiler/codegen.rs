use {
    super::{
        super::{
            super::frontend::lexer::{DataTypes, TokenKind},
            instruction::Instruction,
        },
        binaryop, call,
        objects::CompilerObjects,
        types::Function,
        unaryop, utils, variable,
    },
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        types::FunctionType,
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

            Instruction::Free {
                name,
                is_string,
                free_only,
            } => {
                let var: PointerValue<'ctx> = self.compiler_objects.find_and_get(name).unwrap();

                if *is_string && !free_only {
                    self.builder
                        .build_call(
                            self.module.get_function("Vec.destroy").unwrap(),
                            &[var.into()],
                            "",
                        )
                        .unwrap();
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

                self.codegen(variable.as_ref().unwrap());

                let start_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(start_block);

                let cond: IntValue<'ctx> = self
                    .codegen(cond.as_ref().unwrap())
                    .as_basic_value()
                    .into_int_value();

                let then_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");
                let exit_block: BasicBlock<'ctx> = self.context.append_basic_block(function, "");

                self.builder
                    .build_conditional_branch(cond, then_block, exit_block)
                    .unwrap();

                self.builder.position_at_end(then_block);

                self.codegen(actions.as_ref().unwrap());
                self.codegen(block.as_ref());

                self.builder
                    .build_unconditional_branch(start_block)
                    .unwrap();

                self.builder.position_at_end(exit_block);

                Instruction::Null
            }

            Instruction::Extern { .. } => Instruction::Null,

            Instruction::Function {
                name,
                params,
                body,
                return_kind,
                is_public,
            } => {
                if let Some(body) = body {
                    self.compile_function(
                        (name, params, Some(body), return_kind, *is_public),
                        false,
                    );
                    return Instruction::Null;
                }

                Instruction::Null
            }

            Instruction::Return(instr, kind) => {
                self.emit_return(instr, kind);
                Instruction::Null
            }

            Instruction::String(_) => Instruction::BasicValueEnum(build_basic_value_enum(
                self.module,
                self.builder,
                self.context,
                instr,
                false,
                &self.compiler_objects,
            )),

            Instruction::Var {
                name,
                kind,
                value,
                only_comptime,
                ..
            } => {
                if *only_comptime {
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

            Instruction::Indexe {
                origin: origin_name,
                index,
                ..
            } => {
                let variable: PointerValue<'ctx> =
                    self.compiler_objects.find_and_get(origin_name).unwrap();

                let value: IntValue<'_> = self
                    .builder
                    .build_call(
                        self.module.get_function("Vec.get_i8").unwrap(),
                        &[
                            variable.into(),
                            self.context.i64_type().const_int(*index, false).into(),
                        ],
                        "",
                    )
                    .unwrap()
                    .try_as_basic_value()
                    .unwrap_left()
                    .into_int_value();

                let char: PointerValue<'_> = self.emit_char_from_indexe(value);

                Instruction::BasicValueEnum(char.into())
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

    fn emit_return(&mut self, instr: &'ctx Instruction, kind: &DataTypes) {
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

        if let Instruction::Indexe { origin, index, .. } = instr {
            let var: PointerValue<'ctx> = self.compiler_objects.find_and_get(origin).unwrap();

            let char: IntValue<'_> = self
                .builder
                .build_call(
                    self.module.get_function("Vec.get_i8").unwrap(),
                    &[
                        var.into(),
                        self.context.i64_type().const_int(*index, false).into(),
                    ],
                    "",
                )
                .unwrap()
                .try_as_basic_value()
                .unwrap_left()
                .into_int_value();

            self.builder.build_return(Some(&char)).unwrap();

            return;
        }

        if let Instruction::String(_) = instr {
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
            if let DataTypes::String = kind {
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

    fn compile_external(&mut self, external_name: &str, instr: &'ctx Instruction) {
        let function: Function = instr.as_function();

        let kind: FunctionType<'_> =
            utils::datatype_to_fn_type(self.context, function.3, function.1);

        let llvm_function: FunctionValue<'_> =
            self.module
                .add_function(external_name, kind, Some(Linkage::External));

        self.compiler_objects
            .insert_function(function.0, llvm_function);
    }

    fn compile_function(&mut self, function: Function<'ctx>, only_define: bool) {
        let function_name: &str = function.0;
        let function_return_type: &Option<DataTypes> = function.3;
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

            let mut index: usize = 0;

            function.get_params().iter().for_each(|param| {
                if let Some(Instruction::Param { name, .. }) = function_params.get(index) {
                    param.set_name(name);
                }

                index += 1;
            });

            self.function = Some(function);

            self.compiler_objects
                .insert_function(function_name, function);

            return;
        }

        let function: FunctionValue<'ctx> = self.module.get_function(function_name).unwrap();

        let entry: BasicBlock = self.context.append_basic_block(function, "");

        self.builder.position_at_end(entry);

        self.codegen(function_body.unwrap());

        if function_return_type.is_none() {
            self.builder.build_return(None).unwrap();
        }
    }

    fn emit_char_from_indexe(&mut self, value: IntValue<'ctx>) -> PointerValue<'ctx> {
        let char: PointerValue<'ctx> = self
            .builder
            .build_alloca(self.context.i8_type(), "")
            .unwrap();

        self.builder.build_store(char, value).unwrap();

        char
    }

    fn predefine_functions(&mut self) {
        self.instructions.iter().for_each(|instr| {
            if instr.is_function() {
                let function: Function = instr.as_function();

                self.compile_function(function, true);
            } else if instr.is_extern() {
                let external: (&str, &Instruction, TokenKind) = instr.as_extern();

                // CHECK IF TOKENKIND IS FUNCTION KIND, (REMEMBER)

                self.compile_external(external.0, external.1);
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

pub fn build_basic_value_enum<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instr: &'ctx Instruction,
    is_var: bool,
    objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::String(str) = instr {
        if !is_var {
            return utils::build_string_constant(module, builder, context, str).into();
        }

        return utils::build_dynamic_string(module, builder, context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed) = instr {
        return utils::build_const_float(builder, context, kind, *num, *is_signed).into();
    }

    if let Instruction::Integer(kind, num, is_signed) = instr {
        return utils::build_const_integer(context, kind, *num as u64, *is_signed).into();
    }

    if let Instruction::Char(char) = instr {
        return context.i8_type().const_int(*char as u64, false).into();
    }

    if let Instruction::Boolean(bool) = instr {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::RefVar { name, kind, .. } = instr {
        let var: PointerValue<'ctx> = objects.find_and_get(name).unwrap();

        if kind.is_float() {
            return builder
                .build_load(utils::datatype_float_to_llvm_type(context, kind), var, "")
                .unwrap();
        }

        if kind.is_integer() || *kind == DataTypes::Bool {
            return builder
                .build_load(utils::datatype_integer_to_llvm_type(context, kind), var, "")
                .unwrap();
        }

        if *kind == DataTypes::String {
            return builder
                .build_call(module.get_function("Vec.clone").unwrap(), &[var.into()], "")
                .unwrap()
                .try_as_basic_value()
                .unwrap_left();
        }

        unreachable!()
    }

    unreachable!()
}
