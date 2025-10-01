#![allow(clippy::collapsible_if)]

use std::path::PathBuf;

use crate::backends::classical::llvm::compiler::declarations::{self};
use crate::backends::classical::llvm::compiler::statements;
use crate::backends::classical::llvm::compiler::{self, builtins};
use crate::backends::classical::llvm::compiler::{abort, ptr};
use crate::backends::classical::llvm::compiler::{binaryop, generation};
use crate::backends::classical::llvm::compiler::{block, typegen};

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::metadata::local::LocalMetadata;
use crate::frontends::classical::typesystem::traits::LLVMTypeExtensions;
use crate::frontends::classical::typesystem::types::Type;

use super::{context::LLVMCodeGenContext, value};

use inkwell::basic_block::BasicBlock;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::FunctionType,
    values::{BasicValueEnum, FunctionValue},
};

#[derive(Debug)]
pub struct LLVMCodegen<'a, 'ctx> {
    context: &'a mut LLVMCodeGenContext<'a, 'ctx>,
    ast: &'ctx [Ast<'ctx>],
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    pub fn generate(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        Self { context, ast }.compile();
    }

    fn compile(&mut self) {
        self.declare_forward();

        self.ast.iter().for_each(|ast| {
            self.codegen(ast);
        });
    }

    fn codegen(&mut self, decl: &'ctx Ast) {
        self.codegen_declaration(decl);
    }

    pub fn codegen_declaration(&mut self, decl: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | DECLARATIONS - START


        ########################################################################*/

        match decl {
            Ast::EntryPoint {
                body, parameters, ..
            } => {
                self.build_entrypoint(parameters, body);
            }

            Ast::Function { body, .. } => {
                if body.is_none() {
                    return;
                }

                declarations::function::compile_body(self, decl.as_global_function());
            }

            Ast::GlobalAssembler { asm, .. } => {
                self.context.get_llvm_module().set_inline_assembly(asm);
            }

            _ => (),
        }

        /* ######################################################################


            LLVM CODEGEN | DECLARATIONS - END


        ########################################################################*/
    }

    pub fn codegen_block(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - START


        ########################################################################*/

        match stmt {
            Ast::Block { stmts, .. } => {
                self.context.begin_scope();

                stmts.iter().for_each(|stmt| {
                    self.codegen_block(stmt);
                });

                self.context.end_scope();

                block::move_terminator_to_end(self.get_context());
            }

            stmt => self.stmt(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - END


        ########################################################################*/
    }

    fn stmt(&mut self, stmt: &'ctx Ast) {
        self.codegen_conditionals(stmt);
    }

    pub fn codegen_conditionals(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - START


        ########################################################################*/

        match stmt {
            Ast::If { .. } => statements::conditional::compile(self, stmt),

            stmt => self.codegen_loops(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - END


        ########################################################################*/
    }

    pub fn codegen_loops(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOPS - START


        ########################################################################*/

        match stmt {
            // Loops
            Ast::While { .. } => statements::loops::whileloop::compile(self, stmt),
            Ast::Loop { .. } => statements::loops::infloop::compile(self, stmt),
            Ast::For { .. } => statements::loops::forloop::compile(self, stmt),

            // Control Flow
            Ast::Break { span } => {
                let llvm_builder: &Builder = self.context.get_llvm_builder();

                let break_block: BasicBlock = self.context.get_loop_ctx().get_last_break_branch();

                llvm_builder
                    .build_unconditional_branch(break_block)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'break' loop control flow!",
                            *span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }
            Ast::Continue { span } => {
                let llvm_builder: &Builder = self.context.get_llvm_builder();

                let continue_block: BasicBlock =
                    self.context.get_loop_ctx().get_last_continue_branch();

                llvm_builder
                    .build_unconditional_branch(continue_block)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'continue' loop control flow!",
                            *span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }

            stmt => self.codegen_variables(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOPS - END


        ########################################################################*/
    }

    pub fn codegen_variables(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | VARIABLES - START


        ########################################################################*/

        match stmt {
            Ast::Local { metadata, .. } => {
                let metadata: &LocalMetadata = metadata;

                if metadata.is_undefined() {
                    self.context.new_local(stmt.as_local());

                    return;
                }

                statements::local::compile(self.context, stmt.as_local());
            }

            Ast::Const { .. } => {
                self.context.new_local_constant(stmt.as_local_constant());
            }

            Ast::Static { .. } => {
                self.context.new_local_static(stmt.as_local_static());
            }

            Ast::LLI {
                name,
                kind,
                expr,
                span,
                ..
            } => {
                statements::lli::compile(self.context, name, kind, expr, *span);
            }

            stmt => self.codegen_terminator(stmt),
        }

        /* ######################################################################


            LLVM CODEGEN | VARIABLES - END


        ########################################################################*/
    }

    pub fn codegen_terminator(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - START


        ########################################################################*/

        match stmt {
            Ast::Return {
                expression,
                kind,
                span,
                ..
            } => {
                let llvm_builder: &Builder = self.context.get_llvm_builder();

                if expression.is_none() {
                    if llvm_builder.build_return(None).is_err() {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'return'!",
                            *span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    }
                }

                if let Some(expr) = expression {
                    if llvm_builder
                        .build_return(Some(&self::compile_expr(self.context, expr, Some(kind))))
                        .is_err()
                    {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'return'!",
                            *span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    }
                }
            }

            any => self.expressions(any),
        }

        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - END


        ########################################################################*/
    }

    fn expressions(&mut self, stmt: &'ctx Ast) {
        self.codegen_loose(stmt);
    }

    pub fn codegen_loose(&mut self, stmt: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS || STATEMENTS - START


        ########################################################################*/

        match stmt {
            Ast::UnaryOp {
                operator,
                kind,
                expression,
                ..
            } => {
                generation::expressions::unary::compile(
                    self.context,
                    (operator, kind, expression),
                    None,
                );
            }

            Ast::BinaryOp {
                left,
                operator,
                right,
                kind,
                span,
            } => {
                if kind.is_integer_type() {
                    binaryop::integer::compile(self.context, (left, operator, right, *span), None);
                    return;
                }

                if kind.is_float_type() {
                    binaryop::float::compile(self.context, (left, operator, right, *span), None);
                    return;
                }

                if kind.is_bool_type() {
                    binaryop::boolean::compile(self.context, (left, operator, right, *span), None);
                    return;
                }

                abort::abort_codegen(
                    self.context,
                    "Failed to compile binary operation!",
                    *span,
                    PathBuf::from(file!()),
                    line!(),
                )
            }

            Ast::Mut { .. } => {
                statements::mutation::compile(self.context, stmt);
            }

            Ast::Write { .. } => {
                value::compile(self.context, stmt, None);
            }

            Ast::Call { .. } => {
                value::compile(self.context, stmt, None);
            }

            Ast::Indirect { .. } => {
                value::compile(self.context, stmt, None);
            }

            Ast::AsmValue { .. } => {
                value::compile(self.context, stmt, None);
            }

            Ast::Builtin { builtin, .. } => {
                builtins::compile(self.context, builtin, None);
            }

            Ast::Unreachable { .. } => {
                let _ = self.context.get_llvm_builder().build_unreachable();
            }

            _ => (),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS || STATEMENTS - END


        ########################################################################*/
    }

    fn build_entrypoint(&mut self, parameters: &'ctx [Ast], body: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_context: &Context = self.context.get_llvm_context();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let entrypoint_type: FunctionType =
            typegen::generate_fn_type(self.context, &Type::U32, parameters, false);

        let entrypoint: FunctionValue = llvm_module.add_function("main", entrypoint_type, None);

        llvm_builder.position_at_end(block::append_block(llvm_context, entrypoint));

        self.context.set_current_fn(entrypoint);

        parameters.iter().for_each(|parameter| {
            compiler::declarations::function::compile_parameter(
                self,
                entrypoint,
                parameter.as_function_parameter(),
            );
        });

        self.codegen_block(body);
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | START


    ########################################################################*/

    fn declare_forward(&mut self) {
        self.ast.iter().for_each(|ast| match ast {
            Ast::AssemblerFunction { .. } => {
                declarations::asmfunction::compile(self.context, ast.as_global_asm_function())
            }
            Ast::Function { .. } => {
                declarations::function::compile_decl(self.context, ast.as_global_function())
            }

            Ast::Const { .. } => {
                self.context.new_global_constant(ast.as_global_constant());
            }
            Ast::Static { .. } => {
                self.context.new_global_static(ast.as_global_static());
            }

            _ => (),
        });
    }

    /* ######################################################################


        CODEGEN FORWARD DECLARATION | END


    ########################################################################*/
}

pub fn compile_expr<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let expr_type: &Type = expr.get_type_unwrapped();

    if expr_type.llvm_is_ptr_type() {
        return ptr::compile(context, expr, cast_type);
    }

    value::compile(context, expr, cast_type)
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    #[inline]
    pub fn get_mut_context(&mut self) -> &mut LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }

    #[inline]
    pub fn get_context(&self) -> &LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}
