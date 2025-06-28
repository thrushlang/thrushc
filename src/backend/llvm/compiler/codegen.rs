#![allow(clippy::collapsible_if)]

use crate::backend::llvm::compiler::{builtins, lli, mutation, ptrgen, utils};
use crate::backend::types::{repr::LLVMFunction, traits::AssemblerFunctionExtensions};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::parser::repr::{
    AssemblerFunctionRepresentation, FunctionParameter, FunctionRepresentation,
};
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;

use crate::frontend::types::ast::Ast;

use super::super::compiler::attributes::LLVMAttribute;

use super::{
    attributes::{AttributeBuilder, LLVMAttributeApplicant},
    context::LLVMCodeGenContext,
    conventions::CallConvention,
    local, typegen, valuegen,
};

use inkwell::InlineAsmDialect;
use inkwell::values::{BasicMetadataValueEnum, PointerValue};
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
    values::{BasicValueEnum, FunctionValue, IntValue},
};

pub struct LLVMCodegen<'a, 'ctx> {
    context: &'a mut LLVMCodeGenContext<'a, 'ctx>,
    ast: &'ctx [Ast<'ctx>],
    current_function: Option<FunctionValue<'ctx>>,
    exit_loop_block: Option<BasicBlock<'ctx>>,
    start_loop_block: Option<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    pub fn generate(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        Self {
            context,
            ast,
            current_function: None,
            exit_loop_block: None,
            start_loop_block: None,
        }
        .start();
    }

    fn start(&mut self) {
        self.init_asm_functions();
        self.init_functions();

        self.ast.iter().for_each(|stmt| {
            self.codegen(stmt);
        });
    }

    fn codegen(&mut self, stmt: &'ctx Ast) {
        self.codegen_function_parts(stmt);
    }

    fn codegen_function_parts(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | FUNCTIONS - START


        ########################################################################*/

        let llvm_builder: &Builder = self.context.get_llvm_builder();
        let llvm_context: &Context = self.context.get_llvm_context();

        match stmt {
            Ast::EntryPoint { body, .. } => {
                self.current_function = Some(self.entrypoint());

                self.codegen(body);

                if llvm_builder
                    .build_return(Some(&llvm_context.i32_type().const_int(0, false)))
                    .is_err()
                {
                    logging::log(
                        LoggingType::BackendPanic,
                        "Unable to build the return of entrypoint.",
                    );
                }
            }

            Ast::Function { .. } => {
                self.compile_function(stmt.as_function_representation());
            }

            Ast::Return {
                expression, kind, ..
            } => {
                if expression.is_none() {
                    if llvm_builder.build_return(None).is_err() {
                        {
                            logging::log(
                                LoggingType::BackendPanic,
                                "Unable to build the return instruction at code generation time.",
                            );
                        }
                    }
                }

                if let Some(expression) = expression {
                    if kind.is_ptr_type() || kind.is_mut_type() {
                        if llvm_builder
                            .build_return(Some(&ptrgen::compile(
                                self.context,
                                expression,
                                Some(kind),
                            )))
                            .is_err()
                        {
                            {
                                logging::log(
                                    LoggingType::BackendPanic,
                                    "Unable to build the return instruction at code generation time.",
                                );
                            }
                        };
                    } else if llvm_builder
                        .build_return(Some(&valuegen::compile(
                            self.context,
                            expression,
                            Some(kind),
                        )))
                        .is_err()
                    {
                        {
                            logging::log(
                                LoggingType::BackendPanic,
                                "Unable to build the return instruction at code generation time.",
                            );
                        }
                    };
                }
            }

            stmt => self.codegen_code_block(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | FUNCTIONS - END


        ########################################################################*/
    }

    fn codegen_code_block(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - START


        ########################################################################*/

        match stmt {
            Ast::Block { stmts, .. } => {
                self.context.begin_scope();

                stmts.iter().for_each(|stmt| {
                    self.codegen(stmt);
                });

                self.context.end_scope();
            }

            stmt => self.codegen_conditionals(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - END


        ########################################################################*/
    }

    fn codegen_conditionals(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - START


        ########################################################################*/

        let llvm_builder: &Builder = self.context.get_llvm_builder();
        let llvm_context: &Context = self.context.get_llvm_context();

        match stmt {
            Ast::If {
                cond,
                block,
                elfs,
                otherwise,
                ..
            } => {
                if let Some(current_function) = self.current_function {
                    let if_comparison: IntValue<'ctx> =
                        valuegen::compile(self.context, cond, Some(&ThrushType::Bool))
                            .into_int_value();

                    let then_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "if");

                    let else_if_cond: BasicBlock =
                        llvm_context.append_basic_block(current_function, "elseif");

                    let else_if_body: BasicBlock =
                        llvm_context.append_basic_block(current_function, "elseif_body");

                    let else_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "else");

                    let merge_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "merge");

                    if !elfs.is_empty() {
                        llvm_builder
                            .build_conditional_branch(if_comparison, then_block, else_if_cond)
                            .unwrap();
                    } else if otherwise.is_some() && elfs.is_empty() {
                        llvm_builder
                            .build_conditional_branch(if_comparison, then_block, else_block)
                            .unwrap();
                    } else {
                        llvm_builder
                            .build_conditional_branch(if_comparison, then_block, merge_block)
                            .unwrap();
                    }

                    llvm_builder.position_at_end(then_block);

                    self.codegen(block);

                    if !block.has_return() && !block.has_break() && !block.has_continue() {
                        llvm_builder
                            .build_unconditional_branch(merge_block)
                            .unwrap();
                    }

                    if !elfs.is_empty() {
                        llvm_builder.position_at_end(else_if_cond);
                    } else {
                        llvm_builder.position_at_end(merge_block);
                    }

                    if !elfs.is_empty() {
                        let mut current_block: BasicBlock = else_if_body;

                        for (index, instr) in elfs.iter().enumerate() {
                            if let Ast::Elif { cond, block, .. } = instr {
                                let compiled_else_if_cond: IntValue =
                                    valuegen::compile(self.context, cond, Some(&ThrushType::Bool))
                                        .into_int_value();

                                let elif_body: BasicBlock = current_block;

                                let next_block: BasicBlock = if index + 1 < elfs.len() {
                                    llvm_context.append_basic_block(current_function, "elseif_body")
                                } else if otherwise.is_some() {
                                    else_block
                                } else {
                                    merge_block
                                };

                                llvm_builder
                                    .build_conditional_branch(
                                        compiled_else_if_cond,
                                        elif_body,
                                        next_block,
                                    )
                                    .unwrap();

                                llvm_builder.position_at_end(elif_body);

                                self.codegen(block);

                                if !block.has_return()
                                    && !block.has_break()
                                    && !block.has_continue()
                                {
                                    llvm_builder
                                        .build_unconditional_branch(merge_block)
                                        .unwrap();
                                }

                                if index + 1 < elfs.len() {
                                    llvm_builder.position_at_end(next_block);
                                    current_block = llvm_context
                                        .append_basic_block(current_function, "elseif_body");
                                }
                            }
                        }
                    }

                    if let Some(otherwise) = otherwise {
                        if let Ast::Else { block, .. } = &**otherwise {
                            llvm_builder.position_at_end(else_block);

                            self.codegen(block);

                            if !block.has_return() && !block.has_break() && !block.has_continue() {
                                llvm_builder
                                    .build_unconditional_branch(merge_block)
                                    .unwrap();
                            }
                        }
                    }

                    if !elfs.is_empty() || otherwise.is_some() {
                        llvm_builder.position_at_end(merge_block);
                    }

                    if elfs.is_empty() {
                        let _ = else_if_cond.remove_from_function();
                        let _ = else_if_body.remove_from_function();
                    }

                    if otherwise.is_none() {
                        let _ = else_block.remove_from_function();
                    }

                    return;
                }

                logging::log(
                    LoggingType::BackendPanic,
                    "The current function could not be obtained at code generation time.",
                );
            }
            stmt => self.codegen_loops(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - END


        ########################################################################*/
    }

    fn codegen_loops(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOPS - START


        ########################################################################*/

        let llvm_builder: &Builder = self.context.get_llvm_builder();
        let llvm_context: &Context = self.context.get_llvm_context();

        match stmt {
            Ast::While { cond, block, .. } => {
                if let Some(current_function) = self.current_function {
                    let condition_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "while");

                    llvm_builder
                        .build_unconditional_branch(condition_block)
                        .unwrap();

                    llvm_builder.position_at_end(condition_block);

                    let conditional: IntValue =
                        valuegen::compile(self.context, cond, Some(&ThrushType::Bool))
                            .into_int_value();

                    let then_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "while_body");
                    let exit_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "while_exit");

                    self.exit_loop_block = Some(exit_block);

                    llvm_builder
                        .build_conditional_branch(conditional, then_block, exit_block)
                        .unwrap();

                    llvm_builder.position_at_end(then_block);

                    self.codegen(block);

                    let exit_brancher = llvm_builder
                        .build_unconditional_branch(condition_block)
                        .unwrap();

                    if block.has_break() {
                        exit_brancher.remove_from_basic_block();
                    }

                    llvm_builder.position_at_end(exit_block);

                    return;
                }

                logging::log(
                    LoggingType::BackendPanic,
                    "The current function could not be obtained at code generation time.",
                );
            }
            Ast::Loop { block, .. } => {
                if let Some(function) = self.current_function {
                    let start_loop_block: BasicBlock =
                        llvm_context.append_basic_block(function, "loop");

                    llvm_builder
                        .build_unconditional_branch(start_loop_block)
                        .unwrap();

                    llvm_builder.position_at_end(start_loop_block);

                    let exit_loop_block: BasicBlock =
                        llvm_context.append_basic_block(function, "loop_exit");

                    self.exit_loop_block = Some(exit_loop_block);

                    self.codegen(block);

                    if !block.has_return() && !block.has_break() && !block.has_continue() {
                        let _ = exit_loop_block.remove_from_function();

                        llvm_builder
                            .build_unconditional_branch(function.get_last_basic_block().unwrap())
                            .unwrap();
                    } else {
                        llvm_builder.position_at_end(exit_loop_block);
                    }

                    return;
                }

                logging::log(
                    LoggingType::BackendPanic,
                    "The current function could not be obtained at code generation time.",
                );
            }
            Ast::For {
                local,
                cond,
                actions,
                block,
                ..
            } => {
                if let Some(current_function) = self.current_function {
                    self.codegen(local.as_ref());

                    let start_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "for");

                    self.start_loop_block = Some(start_block);

                    llvm_builder
                        .build_unconditional_branch(start_block)
                        .unwrap();

                    llvm_builder.position_at_end(start_block);

                    let condition: IntValue =
                        valuegen::compile(self.context, cond, Some(&ThrushType::Bool))
                            .into_int_value();

                    let then_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "for_body");
                    let exit_block: BasicBlock =
                        llvm_context.append_basic_block(current_function, "for_exit");

                    llvm_builder
                        .build_conditional_branch(condition, then_block, exit_block)
                        .unwrap();

                    self.exit_loop_block = Some(exit_block);

                    llvm_builder.position_at_end(then_block);

                    if actions.is_pre_unaryop() {
                        self.codegen(block.as_ref());

                        let _ = valuegen::compile(self.context, actions, None);
                    } else {
                        let _ = valuegen::compile(self.context, actions, None);

                        self.codegen(block.as_ref());
                    }

                    let exit_brancher = llvm_builder
                        .build_unconditional_branch(start_block)
                        .unwrap();

                    if block.has_break() {
                        exit_brancher.remove_from_basic_block();
                    }

                    llvm_builder.position_at_end(exit_block);

                    return;
                }

                logging::log(
                    LoggingType::BackendPanic,
                    "The current function could not be obtained at code generation time.",
                );
            }

            Ast::Break { .. } => {
                llvm_builder
                    .build_unconditional_branch(self.exit_loop_block.unwrap())
                    .unwrap();
            }

            Ast::Continue { .. } => {
                llvm_builder
                    .build_unconditional_branch(self.start_loop_block.unwrap())
                    .unwrap();
            }

            stmt => self.codegen_variables(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOPS - END


        ########################################################################*/
    }

    fn codegen_variables(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | VARIABLES - START


        ########################################################################*/

        match stmt {
            Ast::Local {
                name,
                ascii_name,
                kind,
                value,
                attributes,
                undefined,
                ..
            } => {
                if *undefined {
                    self.context.alloc_local(name, ascii_name, kind, attributes);
                    return;
                }

                local::new((name, ascii_name, kind, value, attributes), self.context);
            }

            Ast::Const {
                name,
                kind,
                value,
                attributes,
                ..
            } => {
                let value: BasicValueEnum = valuegen::compile(self.context, value, Some(kind));
                self.context.alloc_constant(name, kind, value, attributes);
            }

            Ast::LLI {
                name, kind, value, ..
            } => {
                lli::new(name, kind, value, self.context);
            }

            stmt => self.codegen_loose_expression(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | VARIABLES - END


        ########################################################################*/
    }

    fn codegen_loose_expression(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS - START


        ########################################################################*/

        match stmt {
            Ast::Mut { .. } => {
                mutation::compile(self.context, stmt);
            }

            Ast::Write { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::Call { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::AsmValue { .. } => {
                valuegen::compile(self.context, stmt, None);
            }

            Ast::Builtin { builtin, .. } => {
                builtins::compile(self.context, builtin, None);
            }

            _ => (),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS - END


        ########################################################################*/
    }

    fn entrypoint(&mut self) -> FunctionValue<'ctx> {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let main_type: FunctionType = llvm_context.i32_type().fn_type(&[], false);
        let main: FunctionValue = llvm_module.add_function("main", main_type, None);

        let main_block: BasicBlock = llvm_context.append_basic_block(main, "");

        llvm_builder.position_at_end(main_block);

        main
    }

    fn compile_function_parameter(
        &mut self,
        llvm_function: FunctionValue<'ctx>,
        parameter: FunctionParameter<'ctx>,
    ) {
        let parameter_name: &str = parameter.0;
        let parameter_ascii_name: &str = parameter.1;
        let parameter_type: &ThrushType = parameter.2;
        let parameter_position: u32 = parameter.3;

        if let Some(raw_value_llvm_parameter) = llvm_function.get_nth_param(parameter_position) {
            raw_value_llvm_parameter.set_name(parameter_ascii_name);

            self.context.alloc_function_parameter(
                parameter_name,
                parameter_type,
                raw_value_llvm_parameter,
            );
        } else {
            logging::log(
                LoggingType::BackendPanic,
                "The value of a parameter of an LLVM function could not be obtained at code generation time.",
            );
        }
    }

    fn compile_function(&mut self, function: FunctionRepresentation<'ctx>) {
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let function_ascii_name: &str = function.1;
        let function_type: &ThrushType = function.2;
        let function_parameters: &[Ast<'ctx>] = function.3;
        let function_body: &Ast = function.5;

        if function_body.is_null() {
            return;
        }

        let get_llvm_function: LLVMFunction = self.context.get_function(function_ascii_name);
        let llvm_function_value: FunctionValue = get_llvm_function.0;

        let llvm_function: FunctionValue = llvm_function_value;

        let entry: BasicBlock = llvm_context.append_basic_block(llvm_function, "");

        llvm_builder.position_at_end(entry);

        self.current_function = Some(llvm_function);

        function_parameters.iter().for_each(|parameter| {
            if let Ast::FunctionParameter {
                name,
                ascii_name,
                kind,
                position,
                is_mutable,
                ..
            } = parameter
            {
                self.compile_function_parameter(
                    llvm_function,
                    (name, ascii_name, kind, *position, *is_mutable),
                );
            }
        });

        self.codegen(function_body);

        if !function_body.has_return() && function_type.is_void_type() {
            llvm_builder.build_return(None).unwrap();
        }

        self.current_function = None;
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | START


    ########################################################################*/

    fn init_functions(&mut self) {
        self.ast.iter().for_each(|stmt| {
            if stmt.is_function() {
                self.declare_function(stmt);
            }
        });
    }

    fn init_asm_functions(&mut self) {
        self.ast.iter().for_each(|stmt| {
            if stmt.is_asm_function() {
                self.compile_asm_function(stmt);
            }
        });
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | END


    ########################################################################*/

    fn compile_asm_function(&mut self, stmt: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let last_builder_block: Option<BasicBlock> = llvm_builder.get_insert_block();

        let asm_function: AssemblerFunctionRepresentation = stmt.as_asm_function_representation();

        let asm_function_name: &str = asm_function.0;
        let asm_function_ascii_name: &str = asm_function.1;
        let asm_function_assembler: String = asm_function.2.to_string();
        let asm_function_constraints: String = asm_function.3.to_string();
        let asm_function_return_type: &ThrushType = asm_function.4;
        let asm_function_parameters: &[Ast] = asm_function.5;
        let asm_function_parameters_types: &[ThrushType] = asm_function.6;
        let asm_function_attributes: &ThrushAttributes = asm_function.7;

        let mut call_convention: u32 = CallConvention::Standard as u32;

        let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;
        let sideeffects: bool = asm_function_attributes.has_asmsideffects_attribute();
        let align_stack: bool = asm_function_attributes.has_asmalignstack_attribute();
        let can_throw: bool = asm_function_attributes.has_asmthrow_attribute();
        let is_public: bool = asm_function_attributes.has_public_attribute();

        asm_function_attributes.iter().for_each(|attribute| {
            if let LLVMAttribute::Convention(call_conv, _) = attribute {
                call_convention = (*call_conv) as u32;
            }

            if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attribute {
                syntax = str::assembler_syntax_attr_to_inline_assembler_dialect(new_syntax);
            }
        });

        let truly_function_name: String =
            utils::generate_assembler_function_name(asm_function_ascii_name);

        let asm_function_type: FunctionType = typegen::function_type(
            self.context,
            asm_function_return_type,
            asm_function_parameters,
            false,
        );

        let asm_function_ptr: PointerValue = llvm_context.create_inline_asm(
            asm_function_type,
            asm_function_assembler,
            asm_function_constraints,
            sideeffects,
            align_stack,
            Some(syntax),
            can_throw,
        );

        let llvm_asm_function: FunctionValue =
            llvm_module.add_function(&truly_function_name, asm_function_type, None);

        if !is_public {
            llvm_asm_function.set_linkage(Linkage::LinkerPrivate);
        }

        let entry: BasicBlock = llvm_context.append_basic_block(llvm_asm_function, "");

        llvm_builder.position_at_end(entry);

        let args: Vec<BasicMetadataValueEnum> = llvm_asm_function
            .get_param_iter()
            .map(|param| param.into())
            .collect();

        if let Ok(asm_fn_call) =
            llvm_builder.build_indirect_call(asm_function_type, asm_function_ptr, &args, "")
        {
            match (
                asm_function_return_type.is_void_type(),
                asm_fn_call.try_as_basic_value().left(),
            ) {
                (false, Some(return_value)) => {
                    llvm_builder.build_return(Some(&return_value))
            .map_err(|_| {
                logging::log(
                    LoggingType::BackendPanic,
                    "Failed to create return terminator with value in assembly function generation.",
                );
            })
            .ok();
                }
                _ => {
                    llvm_builder.build_return(None)
            .map_err(|_| {
                logging::log(
                    LoggingType::BackendPanic,
                    "Failed to create void return terminator in assembly function generation.",
                );
            })
            .ok();
                }
            }
        } else {
            logging::log(
                LoggingType::BackendPanic,
                "Unable to create indirect call for call assembly function.",
            );
        }

        if let Some(previous_block) = last_builder_block {
            llvm_builder.position_at_end(previous_block);
        }

        self.context.add_function(
            asm_function_name,
            (
                llvm_asm_function,
                asm_function_parameters_types,
                call_convention,
            ),
        );
    }

    fn declare_function(&mut self, stmt: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();

        let function: FunctionRepresentation = stmt.as_function_representation();

        let function_name: &str = function.0;
        let function_ascii_name: &str = function.1;
        let function_type: &ThrushType = function.2;
        let function_parameters: &[Ast<'ctx>] = function.3;
        let function_parameters_types: &[ThrushType] = function.4;
        let function_attributes: &ThrushAttributes = function.6;

        let ignore_args: bool = function_attributes.has_ignore_attribute();
        let is_public: bool = function_attributes.has_public_attribute();

        let mut extern_name: Option<&str> = None;

        let mut call_convention: u32 = CallConvention::Standard as u32;

        function_attributes
            .iter()
            .for_each(|attribute| match attribute {
                LLVMAttribute::Extern(name, ..) => {
                    extern_name = Some(name);
                }

                LLVMAttribute::Convention(call_conv, _) => {
                    call_convention = (*call_conv) as u32;
                }
                _ => (),
            });

        let llvm_function_name: &str = if let Some(ffi_name) = extern_name {
            ffi_name
        } else {
            function_ascii_name
        };

        let function_type: FunctionType = typegen::function_type(
            self.context,
            function_type,
            function_parameters,
            ignore_args,
        );

        let function: FunctionValue =
            llvm_module.add_function(llvm_function_name, function_type, None);

        let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
            llvm_context,
            function_attributes,
            LLVMAttributeApplicant::Function(function),
        );

        attribute_builder.add_function_attributes(&mut call_convention);

        if !is_public && extern_name.is_none() {
            function.set_linkage(Linkage::LinkerPrivate);
        }

        self.current_function = Some(function);

        self.context.add_function(
            function_name,
            (function, function_parameters_types, call_convention),
        );
    }
}
