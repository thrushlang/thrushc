#![allow(clippy::collapsible_if)]

use std::path::PathBuf;

use crate::back_end::llvm::compiler::declarations::{self};
use crate::back_end::llvm::compiler::generation::{cast, float, integer};
use crate::back_end::llvm::compiler::statements::lli;
use crate::back_end::llvm::compiler::{self, builtins, codegen, constgen};
use crate::back_end::llvm::compiler::{abort, memory};
use crate::back_end::llvm::compiler::{binaryop, generation};
use crate::back_end::llvm::compiler::{block, typegen};
use crate::back_end::llvm::compiler::{ptr, statements};

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::typesystem::types::Type;

use super::context::LLVMCodeGenContext;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::{
    builder::Builder,
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
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    fn compile(&mut self) {
        self.declare_forward();

        self.ast.iter().for_each(|ast| {
            self.codegen(ast);
        });
    }

    fn codegen(&mut self, node: &'ctx Ast) {
        self.codegen_declaration(node);
    }

    fn codegen_declaration(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | DECLARATIONS - START


        ########################################################################*/

        match node {
            Ast::EntryPoint {
                body,
                parameters,
                kind,
                ..
            } => {
                self.build_standard_entrypoint(kind, parameters, body);
            }

            Ast::Function { body, .. } => {
                if body.is_none() {
                    return;
                }

                declarations::function::compile_body(self, node.as_function());
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

    pub fn codegen_block(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - START


        ########################################################################*/

        match node {
            Ast::Block { stmts, .. } => {
                self.context.begin_scope();

                stmts.iter().for_each(|stmt| {
                    self.codegen_block(stmt);
                });

                self.context.end_scope();

                block::move_terminator_to_end(self.get_context());
            }

            node => self.stmt(node),
        }

        /* ######################################################################


            LLVM CODEGEN | CODE BLOCK - END


        ########################################################################*/
    }

    fn stmt(&mut self, node: &'ctx Ast) {
        self.codegen_conditionals(node);
    }

    fn codegen_conditionals(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - START


        ########################################################################*/

        match node {
            Ast::If { .. } => statements::conditional::compile(self, node),

            node => self.codegen_loops(node),
        }

        /* ######################################################################


            LLVM CODEGEN | IF - ELIF - ELSE - END


        ########################################################################*/
    }

    fn codegen_loops(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOPS - START


        ########################################################################*/

        match node {
            // Loops
            Ast::While { .. } => statements::loops::whileloop::compile(self, node),
            Ast::Loop { .. } => statements::loops::infloop::compile(self, node),
            Ast::For { .. } => statements::loops::forloop::compile(self, node),

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

            node => self.codegen_variables(node),
        }

        /* ######################################################################


            LLVM CODEGEN | LOOPS - END


        ########################################################################*/
    }

    pub fn codegen_variables(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | VARIABLES - START


        ########################################################################*/

        match node {
            Ast::Local { metadata, .. } => {
                let metadata: &LocalMetadata = metadata;

                if metadata.is_undefined() {
                    self.context.new_local(node.as_local());

                    return;
                }

                statements::local::compile(self.context, node.as_local());
            }

            Ast::Const { .. } => {
                self.context.new_local_constant(node.as_local_constant());
            }

            Ast::Static { .. } => {
                self.context.new_local_static(node.as_local_static());
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

    fn codegen_terminator(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - START


        ########################################################################*/

        match node {
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
                        .build_return(Some(&self::compile(self.context, expr, Some(kind))))
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

            node => self.expressions(node),
        }

        /* ######################################################################


            LLVM CODEGEN | TERMINATOR - END


        ########################################################################*/
    }

    fn expressions(&mut self, node: &'ctx Ast) {
        self.codegen_loose(node);
    }

    fn codegen_loose(&mut self, node: &'ctx Ast) {
        /* ######################################################################


            LLVM CODEGEN | LOOSE EXPRESSIONS || STATEMENTS - START


        ########################################################################*/

        match node {
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

            Ast::Mut {
                source,
                value,
                span,
                ..
            } => {
                let source_type: &Type = source.llvm_get_type(self.context);

                let ptr: BasicValueEnum = ptr::compile(self.context, source, None);

                let value: BasicValueEnum =
                    codegen::compile(self.context, value, Some(source_type));

                memory::store_anon(self.context, ptr.into_pointer_value(), value, *span);
            }

            Ast::Write { .. } => {
                self::compile(self.context, node, None);
            }

            Ast::Call { .. } => {
                self::compile(self.context, node, None);
            }

            Ast::Indirect { .. } => {
                self::compile(self.context, node, None);
            }

            Ast::AsmValue { .. } => {
                self::compile(self.context, node, None);
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

    fn build_standard_entrypoint(&mut self, kind: &Type, parameters: &'ctx [Ast], body: &'ctx Ast) {
        let llvm_module: &Module = self.context.get_llvm_module();
        let llvm_builder: &Builder = self.context.get_llvm_builder();

        let entrypoint_type: FunctionType =
            typegen::generate_fn_type(self.context, kind, parameters, false);

        let entrypoint: FunctionValue = llvm_module.add_function("main", entrypoint_type, None);

        llvm_builder.position_at_end(block::append_block(self.context, entrypoint));

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
            Ast::Intrinsic { .. } => {
                declarations::intrinsic::compile(self.context, ast.as_intrinsic());
            }

            Ast::AssemblerFunction { .. } => {
                declarations::asmfunction::compile(self.context, ast.as_asm_function())
            }

            Ast::Function { .. } => {
                declarations::function::compile_decl(self.context, ast.as_function())
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

/* ######################################################################


                    COMPILER - EXPRESSION CODEGEN


########################################################################*/

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        // Literal Expressions
        Ast::Float {
            kind,
            value,
            signed,
            span,
            ..
        } => {
            let float: BasicValueEnum =
                float::generate(context, kind, *value, *signed, *span).into();

            cast::try_cast(context, cast_type, kind, float, *span).unwrap_or(float)
        }

        Ast::Integer {
            kind,
            value,
            signed,
            span,
            ..
        } => {
            let integer: BasicValueEnum =
                integer::generate(context, kind, *value, *signed, *span).into();

            cast::try_cast(context, cast_type, kind, integer, *span).unwrap_or(integer)
        }

        Ast::NullPtr { .. } => context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),

        Ast::Str { bytes, .. } => {
            compiler::generation::expressions::string::compile_str(context, bytes).into()
        }

        Ast::Char { byte, .. } => context
            .get_llvm_context()
            .i8_type()
            .const_int(*byte, false)
            .into(),

        Ast::Boolean { value, .. } => context
            .get_llvm_context()
            .bool_type()
            .const_int(*value, false)
            .into(),

        // Function
        // Compiles a function call
        Ast::Call {
            name, args, kind, ..
        } => compiler::generation::expressions::call::compile(context, name, args, kind, cast_type),

        // Function
        // Compiles a indirect function call
        Ast::Indirect {
            function,
            function_type,
            args,
            span,
            ..
        } => compiler::generation::expressions::indirect::compile(
            context,
            function,
            args,
            function_type,
            *span,
            cast_type,
        ),

        // Expressions
        // Compiles a grouped expression (e.g., parenthesized)
        Ast::Group { expression, .. } => self::compile(context, expression, cast_type),

        Ast::BinaryOp {
            left,
            operator,
            right,
            kind: binaryop_type,
            span,
            ..
        } => match binaryop_type {
            t if t.is_float_type() => {
                binaryop::float::compile(context, (left, operator, right, *span), cast_type)
            }
            t if t.is_integer_type() => {
                binaryop::integer::compile(context, (left, operator, right, *span), cast_type)
            }
            t if t.is_bool_type() => {
                binaryop::boolean::compile(context, (left, operator, right, *span), cast_type)
            }

            _ => {
                abort::abort_codegen(
                    context,
                    "Can't be compiled!.",
                    *span,
                    PathBuf::from(file!()),
                    line!(),
                );
            }
        },

        Ast::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } => compiler::generation::expressions::unary::compile(
            context,
            (operator, kind, expression),
            cast_type,
        ),

        // Direct Reference
        Ast::DirectRef { expr, .. } => ptr::compile(context, expr, cast_type),

        // Symbol/Property Access
        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).load(context),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source, indexes, ..
        } => compiler::generation::expressions::property::compile(context, source, indexes),

        // Memory Access Operations
        // Compiles an indexing operation (e.g., array access)
        Ast::Index { source, index, .. } => {
            compiler::generation::expressions::index::compile(context, source, index)
        }

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Defer {
            value,
            kind,
            metadata,
            span,
            ..
        } => {
            let value: BasicValueEnum = ptr::compile(context, value, Some(kind));

            let defer_value: BasicValueEnum = if value.is_pointer_value() {
                memory::deference(
                    context,
                    value.into_pointer_value(),
                    kind,
                    metadata.get_llvm_metadata(),
                    *span,
                )
            } else {
                value
            };

            cast::try_cast(context, cast_type, kind, defer_value, *span).unwrap_or(defer_value)
        }

        // Array Operations
        // Compiles a fixed-size array
        Ast::FixedArray {
            items, kind, span, ..
        } => compiler::generation::expressions::farray::compile(
            context, items, kind, *span, cast_type,
        ),

        // Compiles a dynamic array
        Ast::Array {
            items, kind, span, ..
        } => compiler::generation::expressions::array::compile(
            context, items, kind, *span, cast_type,
        ),

        // Compiles a struct constructor
        Ast::Constructor {
            args, kind, span, ..
        } => compiler::generation::expressions::structure::compile(context, args, kind, *span),

        // Compiles a type cast_type operation
        Ast::As { from, cast, .. } => compiler::generation::cast::compile(context, from, cast),

        // Low-Level Operations
        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => compiler::generation::expressions::inlineasm::compile(
            context,
            assembler,
            constraints,
            args,
            kind,
            attributes,
        ),

        // Enum Value Access
        Ast::EnumValue { value, .. } => {
            let cast_type: &Type = cast_type.unwrap_or(value.llvm_get_type(context));
            constgen::compile(context, value, cast_type)
        }

        // Builtins
        Ast::Builtin { builtin, .. } => builtins::compile(context, builtin, cast_type),

        // Low-Level Instructions
        ast if ast.is_lli() => lli::compile_advanced(context, expr, cast_type),

        // Fallback, Unknown expressions or statements
        what => {
            abort::abort_codegen(
                context,
                "Unknown expression or statement!",
                what.get_span(),
                PathBuf::from(file!()),
                line!(),
            );
        }
    }
}
