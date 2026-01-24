#![allow(clippy::collapsible_if)]

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::{ArrayType, BasicTypeEnum, StructType};
use inkwell::values::{GlobalValue, PointerValue, StructValue};
use inkwell::{builder::Builder, values::BasicValueEnum};
use thrushc_ast::metadata::{ConstantMetadata, LocalMetadata, StaticMetadata};
use thrushc_attributes::ThrushAttributes;
use thrushc_entities::{GlobalConstant, GlobalStatic, LocalConstant, LocalStatic, LocalVariable};
use thrushc_llvm_attributes::LLVMAttributes;
use thrushc_span::Span;

use crate::anchor::PointerAnchor;
use crate::context::LLVMCodeGenContext;
use crate::expressions::unaryop;
use crate::globals::{asmfunction, function, intrinsic};
use crate::memory::SymbolAllocated;
use crate::metadata::LLVMMetadata;
use crate::statements::{conditional, forloop, infloop, whileloop};
use crate::traits::{AstLLVMGetType, LLVMFunctionExtensions};
use crate::{
    abort, block, builtins, cast, codegen, expressions, memory, memstack, memstatic, typegeneration,
};

use thrushc_ast::Ast;
use thrushc_ast::traits::AstCodeLocation;
use thrushc_llvm_builtins::LLVMBuiltin;
use thrushc_typesystem::Type;
use thrushc_typesystem::traits::{DereferenceExtensions, TypeIsExtensions, TypeStructExtensions};

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
        self.init_top_entities();

        self::init_llvm_constructors(self.get_mut_context());
        self::init_llvm_destructors(self.get_mut_context());

        {
            for node in self.ast.iter() {
                self.codegen(node);
            }
        }

        if let Some(dbg_context) = self.get_context().get_debug_context() {
            dbg_context.finalize()
        }

        LLVMMetadata::setup_platform_specific(self.get_context());
    }

    fn init_top_entities(&mut self) {
        {
            for node in self.ast.iter().rev() {
                match node {
                    Ast::Intrinsic { .. } => {
                        intrinsic::compile(
                            self.context,
                            thrushc_entities::intrinsic_from_ast(node),
                        );
                    }
                    Ast::AssemblerFunction { .. } => asmfunction::compile(
                        self.context,
                        thrushc_entities::assembler_function_from_ast(node),
                    ),
                    Ast::Function { .. } => function::compile_top(
                        self.context,
                        thrushc_entities::function_from_ast(node),
                    ),
                    Ast::Const { .. } => {
                        self.get_mut_context()
                            .get_mut_expressions_optimizations()
                            .setup_all_constant_optimizations();

                        let constant: GlobalConstant =
                            thrushc_entities::global_constant_from_ast(node);

                        let name: &str = constant.0;
                        let ascii_name: &str = constant.1;
                        let kind: &Type = constant.2;
                        let value: &Ast = constant.3;
                        let attributes: LLVMAttributes =
                            thrushc_llvm_attributes::into_llvm_attributes(constant.4);
                        let metadata: ConstantMetadata = constant.5;
                        let span: Span = constant.6;

                        let llvm_type: BasicTypeEnum =
                            typegeneration::compile_from(self.get_mut_context(), kind);
                        let value_type: &Type = value.llvm_get_type();

                        let llvm_value: BasicValueEnum =
                            codegen::compile_constant(self.get_mut_context(), value, kind);
                        let value: BasicValueEnum = cast::try_cast_const(
                            self.get_mut_context(),
                            llvm_value,
                            value_type,
                            kind,
                        );

                        let ptr: PointerValue = memstatic::allocate_global_constant(
                            self.get_mut_context(),
                            ascii_name,
                            llvm_type,
                            value,
                            attributes,
                            metadata,
                        );

                        let symbol: SymbolAllocated = SymbolAllocated::new_constant(
                            ptr.into(),
                            kind,
                            value,
                            metadata.get_llvm_metadata(),
                            span,
                        );

                        self.context.add_global_constant(name, symbol);

                        self.context
                            .get_mut_expressions_optimizations()
                            .denegate_all_expression_optimizations();
                    }
                    Ast::Static { .. } => {
                        self.context
                            .get_mut_expressions_optimizations()
                            .denegate_all_expression_optimizations();

                        let static_: GlobalStatic = thrushc_entities::global_static_from_ast(node);

                        let name: &str = static_.0;
                        let ascii_name: &str = static_.1;

                        let kind: &Type = static_.2;
                        let value: Option<&Ast> = static_.3;

                        let attributes: LLVMAttributes =
                            thrushc_llvm_attributes::into_llvm_attributes(static_.4);
                        let metadata: StaticMetadata = static_.5;
                        let span: Span = static_.6;

                        if let Some(value) = value {
                            let value_type: &Type = value.llvm_get_type();
                            let llvm_type: inkwell::types::BasicTypeEnum =
                                typegeneration::compile_from(self.get_mut_context(), kind);

                            let llvm_value: BasicValueEnum =
                                codegen::compile_constant(self.get_mut_context(), value, kind);
                            let value: BasicValueEnum = cast::try_cast_const(
                                self.get_mut_context(),
                                llvm_value,
                                value_type,
                                kind,
                            );

                            let ptr: PointerValue = memstatic::allocate_global_static(
                                self.get_mut_context(),
                                ascii_name,
                                llvm_type,
                                Some(value),
                                attributes,
                                metadata,
                            );

                            let symbol: SymbolAllocated = SymbolAllocated::new_static(
                                ptr.into(),
                                kind,
                                Some(value),
                                metadata.get_llvm_metadata(),
                                span,
                            );

                            self.context.add_global_static(name, symbol);
                        } else {
                            let llvm_type: inkwell::types::BasicTypeEnum =
                                typegeneration::compile_from(self.get_mut_context(), kind);

                            let ptr: PointerValue = memstatic::allocate_global_static(
                                self.get_mut_context(),
                                ascii_name,
                                llvm_type,
                                None,
                                attributes,
                                metadata,
                            );

                            let symbol: SymbolAllocated = SymbolAllocated::new_static(
                                ptr.into(),
                                kind,
                                None,
                                metadata.get_llvm_metadata(),
                                span,
                            );

                            self.context.add_global_static(name, symbol);
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

impl<'a, 'ctx> LLVMCodegen<'a, 'ctx> {
    fn codegen(&mut self, node: &'ctx Ast) {
        self.codegen_declaration(node);
    }

    fn codegen_declaration(&mut self, node: &'ctx Ast) {
        match node {
            Ast::Function { body, .. } if body.is_some() => {
                function::compile_down(self, thrushc_entities::function_from_ast(node));
            }
            Ast::GlobalAssembler { asm, .. } => {
                self.context.get_llvm_module().set_inline_assembly(asm);
            }

            _ => {}
        }
    }

    pub fn codegen_block(&mut self, node: &'ctx Ast) {
        match node {
            Ast::Block { nodes, span, .. } => {
                self.get_mut_context().add_dbg_block_data(*span);

                self.context.begin_scope();

                nodes.iter().for_each(|node| {
                    self.codegen_block(node);
                });

                self.context.end_scope();

                block::move_terminator_to_end(self.get_mut_context(), *span);
            }

            node => self.stmt(node),
        }
    }

    fn stmt(&mut self, node: &'ctx Ast) {
        self.codegen_conditionals(node);
    }

    fn codegen_conditionals(&mut self, node: &'ctx Ast) {
        match node {
            Ast::If { .. } => conditional::compile(self, node),
            node => self.codegen_loops(node),
        }
    }

    fn codegen_loops(&mut self, node: &'ctx Ast) {
        match node {
            // Loops
            Ast::While { .. } => whileloop::compile(self, node),
            Ast::Loop { .. } => infloop::compile(self, node),
            Ast::For { .. } => forloop::compile(self, node),

            // Control Flow
            Ast::Break { span, .. } => {
                self.get_mut_context().mark_dbg_location(*span);

                let llvm_builder: &Builder = self.context.get_llvm_builder();
                let break_block: BasicBlock = self.context.get_loop_ctx().get_last_break_branch();

                llvm_builder
                    .build_unconditional_branch(break_block)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'break' loop control flow!",
                            *span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }
            Ast::BreakAll { span, .. } => {
                self.get_mut_context().mark_dbg_location(*span);

                let llvm_builder: &Builder = self.context.get_llvm_builder();
                let breakall_block: BasicBlock = self.context.get_loop_ctx().get_breakall_branch();

                llvm_builder
                    .build_unconditional_branch(breakall_block)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'breakall' loop control flow!",
                            *span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }
            Ast::Continue { span, .. } => {
                self.get_mut_context().mark_dbg_location(*span);

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
                            std::path::PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }
            Ast::ContinueAll { span, .. } => {
                self.get_mut_context().mark_dbg_location(*span);

                let llvm_builder: &Builder = self.context.get_llvm_builder();
                let continueall_block: BasicBlock =
                    self.context.get_loop_ctx().get_continueall_branch();

                llvm_builder
                    .build_unconditional_branch(continueall_block)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile 'continueall' loop control flow!",
                            *span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }

            node => self.codegen_variables(node),
        }
    }

    pub fn codegen_variables(&mut self, node: &'ctx Ast) {
        match node {
            Ast::Local { metadata, .. } => {
                self.context
                    .get_mut_expressions_optimizations()
                    .denegate_all_expression_optimizations();

                if metadata.is_undefined() {
                    let localvar: LocalVariable = thrushc_entities::local_variable_from_ast(node);

                    let name: &str = localvar.0;
                    let ascii_name: &str = localvar.1;

                    let kind: &Type = localvar.2;

                    let attributes: &ThrushAttributes = localvar.4;
                    let metadata: LocalMetadata = localvar.5;
                    let span: Span = localvar.6;

                    let ptr: PointerValue = memstack::local_variable(
                        self.get_mut_context(),
                        ascii_name,
                        kind,
                        attributes,
                        span,
                    );

                    let symbol: SymbolAllocated =
                        SymbolAllocated::new_local(ptr, kind, metadata.get_llvm_metadata(), span);

                    self.context.add_local_variable(name, symbol);
                } else {
                    let localvar: LocalVariable = thrushc_entities::local_variable_from_ast(node);

                    let name: &str = localvar.0;
                    let ascii_name: &str = localvar.1;

                    let kind: &Type = localvar.2;
                    let value: Option<&Ast> = localvar.3;

                    let attributes: &ThrushAttributes = localvar.4;
                    let metadata: LocalMetadata = localvar.5;
                    let span: Span = localvar.6;

                    let ptr: PointerValue = memstack::local_variable(
                        self.get_mut_context(),
                        ascii_name,
                        kind,
                        attributes,
                        span,
                    );

                    let symbol: SymbolAllocated =
                        SymbolAllocated::new_local(ptr, kind, metadata.get_llvm_metadata(), span);

                    self.context.add_local_variable(name, symbol);

                    let Some(expr) = value else {
                        return;
                    };

                    let symbol: SymbolAllocated = self.context.get_table().get_symbol(name);

                    self.context
                        .set_pointer_anchor(PointerAnchor::new(symbol.get_ptr(), false));

                    let value: BasicValueEnum =
                        codegen::compile(self.get_mut_context(), expr, Some(kind));

                    match self.context.get_pointer_anchor() {
                        Some(anchor) if !anchor.is_triggered() => {
                            symbol.store(self.get_mut_context(), value);
                        }

                        _ => {}
                    }

                    self.context.clear_pointer_anchor();
                }
            }
            Ast::Const { .. } => {
                self.context
                    .get_mut_expressions_optimizations()
                    .setup_all_constant_optimizations();

                let constant: LocalConstant = thrushc_entities::local_constant_from_ast(node);

                let name: &str = constant.0;
                let ascii_name: &str = constant.1;
                let kind: &Type = constant.2;
                let value: &Ast = constant.3;
                let metadata: ConstantMetadata = constant.4;
                let span: Span = constant.5;

                let llvm_type: BasicTypeEnum =
                    typegeneration::compile_from(self.get_mut_context(), kind);
                let value_type: &Type = value.llvm_get_type();

                let llvm_value: BasicValueEnum =
                    codegen::compile_constant(self.get_mut_context(), value, kind);
                let value: BasicValueEnum =
                    cast::try_cast_const(self.get_mut_context(), llvm_value, value_type, kind);

                let ptr: PointerValue = memstatic::allocate_local_constant(
                    self.get_mut_context(),
                    ascii_name,
                    llvm_type,
                    value,
                    metadata,
                );

                let symbol: SymbolAllocated = SymbolAllocated::new_constant(
                    ptr.into(),
                    kind,
                    value,
                    metadata.get_llvm_metadata(),
                    span,
                );

                self.context.add_local_constant(name, symbol);

                self.context
                    .get_mut_expressions_optimizations()
                    .denegate_all_expression_optimizations();
            }
            Ast::Static { .. } => {
                self.context
                    .get_mut_expressions_optimizations()
                    .denegate_all_expression_optimizations();

                let static_: LocalStatic = thrushc_entities::local_static_from_ast(node);

                let name: &str = static_.0;
                let ascii_name: &str = static_.1;

                let kind: &Type = static_.2;
                let value: Option<&Ast> = static_.3;
                let metadata: StaticMetadata = static_.4;
                let span: Span = static_.5;

                if let Some(value) = value {
                    let llvm_type: BasicTypeEnum =
                        typegeneration::compile_from(self.get_mut_context(), kind);
                    let value_type: &Type = value.llvm_get_type();

                    let llvm_value: BasicValueEnum =
                        codegen::compile_constant(self.get_mut_context(), value, kind);
                    let value: BasicValueEnum =
                        cast::try_cast_const(self.get_mut_context(), llvm_value, value_type, kind);

                    let ptr: PointerValue = memstatic::allocate_local_static(
                        self.get_mut_context(),
                        ascii_name,
                        llvm_type,
                        Some(value),
                        metadata,
                    );

                    let symbol: SymbolAllocated = SymbolAllocated::new_static(
                        ptr.into(),
                        kind,
                        Some(value),
                        metadata.get_llvm_metadata(),
                        span,
                    );

                    self.context.add_local_static(name, symbol);
                } else {
                    let llvm_type: BasicTypeEnum =
                        typegeneration::compile_from(self.get_mut_context(), kind);

                    let ptr: PointerValue = memstatic::allocate_local_static(
                        self.get_mut_context(),
                        ascii_name,
                        llvm_type,
                        None,
                        metadata,
                    );

                    let symbol: SymbolAllocated = SymbolAllocated::new_static(
                        ptr.into(),
                        kind,
                        None,
                        metadata.get_llvm_metadata(),
                        span,
                    );

                    self.context.add_local_static(name, symbol);
                }
            }

            stmt => self.codegen_terminator(stmt),
        }
    }

    fn codegen_terminator(&mut self, node: &'ctx Ast) {
        match node {
            Ast::Return {
                expression, span, ..
            } => {
                self.get_mut_context().mark_dbg_location(*span);

                let llvm_builder: &Builder = self.context.get_llvm_builder();

                if expression.is_none() {
                    if llvm_builder.build_return(None).is_err() {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile a function terminator!",
                            *span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    }
                }

                if let Some(expr) = expression {
                    let cast_type: &Type = self
                        .get_mut_context()
                        .get_current_llvm_function(*span)
                        .get_return_type();

                    if llvm_builder
                        .build_return(Some(&self::compile(self.context, expr, Some(cast_type))))
                        .is_err()
                    {
                        abort::abort_codegen(
                            self.context,
                            "Failed to compile a function terminator!",
                            *span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    }
                }
            }

            node => self.expressions(node),
        }
    }

    fn expressions(&mut self, node: &'ctx Ast) {
        self.codegen_loose(node);
    }

    fn codegen_loose(&mut self, node: &'ctx Ast) {
        match node {
            Ast::UnaryOp {
                operator,
                kind,
                expression,
                ..
            } => {
                expressions::unaryop::compile(self.context, (operator, kind, expression), None);
            }

            Ast::BinaryOp {
                left,
                operator,
                right,
                kind,
                span,
            } => {
                if kind.is_integer_type() {
                    expressions::binaryop::integer::compile(
                        self.context,
                        (left, operator, right, *span),
                        None,
                    );
                } else if kind.is_float_type() {
                    expressions::binaryop::float::compile(
                        self.context,
                        (left, operator, right, *span),
                        None,
                    );
                } else if kind.is_bool_type() {
                    expressions::binaryop::boolean::compile(
                        self.context,
                        (left, operator, right, *span),
                        None,
                    );
                } else {
                    abort::abort_codegen(
                        self.context,
                        "Failed to compile binary operation!",
                        *span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                }
            }

            Ast::Mut {
                source,
                value,
                span,
                ..
            } => {
                self.context
                    .get_mut_expressions_optimizations()
                    .denegate_all_expression_optimizations();

                let value_type: &Type = value.llvm_get_type();
                let source_type: &Type = source.llvm_get_type();

                let cast_type: Type = if source_type != value_type {
                    source_type.dereference()
                } else {
                    source_type.clone()
                };

                let ptr: BasicValueEnum = self::compile_as_ptr(self.context, source, None);
                let value: BasicValueEnum = codegen::compile(self.context, value, Some(&cast_type));

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

            Ast::Builtin {
                builtin: thrush_builtin,
                ..
            } => {
                let llvm_builtin: LLVMBuiltin =
                    thrushc_llvm_builtins::into_llvm_builtin(thrush_builtin);

                builtins::compile(self.context, llvm_builtin, None);
            }

            Ast::Unreachable { .. } => {
                let _ = self.context.get_llvm_builder().build_unreachable();
            }

            _ => (),
        }
    }
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
                expressions::floatingpoint::compile(context, kind, *value, *signed, *span).into();

            cast::try_cast(context, cast_type, kind, float, *span)
        }

        Ast::Integer {
            kind,
            value,
            signed,
            span,
            ..
        } => {
            let integer: BasicValueEnum =
                expressions::integer::compile(context, kind, *value, *signed, *span).into();

            cast::try_cast(context, cast_type, kind, integer, *span)
        }

        Ast::NullPtr { .. } => context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),

        Ast::Str { bytes, span, .. } => expressions::cstring::compile(context, bytes, *span).into(),

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
        } => expressions::call::compile(context, name, args, kind, cast_type),

        // Function
        // Compiles a indirect function call
        Ast::Indirect {
            function,
            function_type,
            args,
            span,
            ..
        } => {
            expressions::indirect::compile(context, function, args, function_type, *span, cast_type)
        }

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
            t if t.is_float_type() => expressions::binaryop::float::compile(
                context,
                (left, operator, right, *span),
                cast_type,
            ),
            t if t.is_integer_type() => expressions::binaryop::integer::compile(
                context,
                (left, operator, right, *span),
                cast_type,
            ),
            t if t.is_bool_type() => expressions::binaryop::boolean::compile(
                context,
                (left, operator, right, *span),
                cast_type,
            ),

            _ => {
                abort::abort_codegen(
                    context,
                    "Can't be compiled as binary operation!.",
                    *span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                );
            }
        },

        Ast::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } => expressions::unaryop::compile(context, (operator, kind, expression), cast_type),

        // Direct Reference
        Ast::DirectRef { expr, .. } => self::compile_as_ptr(context, expr, cast_type),

        // Symbol/Property Access
        // Compiles a reference to a variable or symbol
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).load(context),

        // Compiles property access (e.g., struct field or array)
        Ast::Property {
            source, indexes, ..
        } => expressions::property::compile(context, source, indexes),

        // Memory Access Operations
        // Compiles an indexing operation (e.g., array access)
        Ast::Index { source, index, .. } => expressions::index::compile(context, source, index),

        // Compiles a dereference operation (e.g., *pointer)
        Ast::Deref {
            value,
            kind,
            metadata,
            span,
            ..
        } => {
            let value: BasicValueEnum = self::compile_as_ptr(context, value, Some(kind));

            let deref_value: BasicValueEnum = if value.is_pointer_value() {
                memory::dereference(
                    context,
                    value.into_pointer_value(),
                    kind,
                    metadata.get_llvm_metadata(),
                    *span,
                )
            } else {
                value
            };

            cast::try_cast(context, cast_type, kind, deref_value, *span)
        }

        // Array Operations
        // Compiles a fixed-size array
        Ast::FixedArray {
            items, kind, span, ..
        } => expressions::farray::compile(context, items, kind, *span, cast_type),

        // Compiles a dynamic array
        Ast::Array {
            items, kind, span, ..
        } => expressions::array::compile(context, items, kind, *span, cast_type),

        // Compiles a struct constructor
        Ast::Constructor {
            data, kind, span, ..
        } => expressions::structure::compile(context, data, kind, *span),

        // Compiles a type cast_type operation
        Ast::As { from, cast, .. } => cast::compile(context, from, cast),

        // Low-Level Operations
        // Compiles inline assembly code
        Ast::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            span,
            ..
        } => expressions::inlineasm::compile(
            context,
            assembler,
            constraints,
            args,
            kind,
            thrushc_llvm_attributes::into_llvm_attributes(attributes),
            *span,
        ),

        // Enum Value Access
        Ast::EnumValue { value, .. } => {
            let cast_type: &Type = cast_type.unwrap_or(value.llvm_get_type());
            codegen::compile_constant(context, value, cast_type)
        }

        // Builtins
        Ast::Builtin {
            builtin: thrush_builtin,
            ..
        } => {
            let llvm_builtin: LLVMBuiltin =
                thrushc_llvm_builtins::into_llvm_builtin(thrush_builtin);

            builtins::compile(context, llvm_builtin, cast_type)
        }

        // Fallback, Unknown expressions or statements
        what => {
            abort::abort_codegen(
                context,
                "Unknown expression or statement!",
                what.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            );
        }
    }
}

pub fn compile_constant<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ast: &'ctx Ast,
    cast_type: &Type,
) -> BasicValueEnum<'ctx> {
    match ast {
        // Handle integer literals
        Ast::NullPtr { .. } => context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),

        // Character literal compilation
        Ast::Char { byte, .. } => context
            .get_llvm_context()
            .i8_type()
            .const_int(*byte, false)
            .into(),

        // Floating-point constant handling
        Ast::Float {
            value,
            kind,
            signed,
            span,
            ..
        } => {
            let float: BasicValueEnum =
                expressions::floatingpoint::compile(context, kind, *value, *signed, *span).into();

            cast::const_numeric_cast(context, float, cast_type, *signed)
        }

        Ast::Integer {
            value,
            kind,
            signed,
            span,
            ..
        } => {
            let integer: BasicValueEnum =
                expressions::integer::compile(context, kind, *value, *signed, *span).into();

            cast::const_numeric_cast(context, integer, cast_type, *signed)
        }

        // Boolean true/false cases
        Ast::Boolean { value, .. } => context
            .get_llvm_context()
            .bool_type()
            .const_int(*value, false)
            .into(),

        // Fixed-size array
        Ast::FixedArray { items, span, .. } => {
            expressions::farray::compile_const(context, items, cast_type, *span)
        }

        // String literal compilation
        Ast::Str { bytes, span, .. } => expressions::cstring::compile(context, bytes, *span).into(),

        // Struct constructor handling
        Ast::Constructor { data, kind, .. } => {
            let fields_expr: Vec<&Ast> = data.iter().map(|raw_arg| &raw_arg.1).collect();

            let llvm_context: &Context = context.get_llvm_context();

            let struct_fields_types: &[Type] = kind.get_struct_fields();

            let fields: Vec<BasicValueEnum> = fields_expr
                .iter()
                .zip(struct_fields_types)
                .map(|(field, kind)| codegen::compile_constant(context, field, kind))
                .collect();

            llvm_context.const_struct(&fields, false).into()
        }

        // Type cast_typeing operations
        Ast::As { from, cast, .. } => {
            let lhs_type: &Type = from.llvm_get_type();
            let lhs: BasicValueEnum = codegen::compile_constant(context, from, lhs_type);

            cast::try_cast_const(context, lhs, lhs_type, cast)
        }

        // Variable reference resolution
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_value(context),

        // Grouped expression compilation
        Ast::Group { expression, .. } => codegen::compile_constant(context, expression, cast_type),

        // Binary operation dispatch
        Ast::BinaryOp {
            left,
            operator,
            right,
            kind: binaryop_type,
            span,
            ..
        } => {
            if binaryop_type.is_integer_type() {
                return expressions::binaryop::integer::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast_type,
                );
            }

            if binaryop_type.is_bool_type() {
                return expressions::binaryop::boolean::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast_type,
                );
            }

            if binaryop_type.is_float_type() {
                return expressions::binaryop::float::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast_type,
                );
            }

            abort::abort_codegen(
                context,
                "Failed to compile the binary operation!",
                *span,
                std::path::PathBuf::from(file!()),
                line!(),
            );
        }

        // Unary operation dispatch
        Ast::UnaryOp {
            operator,
            expression,
            kind,
            ..
        } => unaryop::compile_const(context, (operator, kind, expression), cast_type),

        // Direct Reference
        Ast::DirectRef { expr, .. } => codegen::compile_as_ptr(context, expr, None),

        // Builtins
        Ast::Builtin { builtin, .. } => {
            let llvm_builtin: LLVMBuiltin<'_> = thrushc_llvm_builtins::into_llvm_builtin(builtin);
            builtins::compile(context, llvm_builtin, Some(cast_type))
        }

        // Enum Value Access
        Ast::EnumValue { value, .. } => codegen::compile_constant(context, value, cast_type),

        // Fallback for unsupported AST nodes
        what => abort::abort_codegen(
            context,
            "Unknown expression or statement!",
            what.get_span(),
            std::path::PathBuf::from(file!()),
            line!(),
        ),
    }
}

pub fn compile_as_ptr<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_ptr().into(),
        _ => codegen::compile(context, expr, cast_type),
    }
}

pub fn init_llvm_constructors<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    if context.get_llvm_ctors().is_empty() {
        return;
    }

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_module: &Module = context.get_llvm_module();

    let ctor_type: StructType = llvm_context.struct_type(
        &[
            llvm_context.i32_type().into(),
            llvm_context.ptr_type(AddressSpace::default()).into(),
            llvm_context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let mut llvm_ctors: Vec<StructValue> = Vec::with_capacity(context.get_llvm_ctors().len());
    let mut last_counter: u32 = 0;

    for (ctor, counter) in context.get_llvm_ctors().iter() {
        if *counter > last_counter {
            let ctor_value: StructValue = ctor_type.const_named_struct(&[
                llvm_context
                    .i32_type()
                    .const_int((*counter).into(), false)
                    .into(),
                (*ctor).into(),
                llvm_context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into(),
            ]);

            llvm_ctors.push(ctor_value);
            last_counter = *counter;
        }
    }

    let actual_size: u32 = u32::try_from(llvm_ctors.len()).unwrap_or(u32::MAX - 1);

    let llvm_ctors_type: ArrayType = ctor_type.array_type(actual_size);
    let global: GlobalValue = llvm_module.add_global(llvm_ctors_type, None, "llvm.global_ctors");

    global.set_linkage(Linkage::Appending);
    global.set_initializer(&ctor_type.const_array(&llvm_ctors));
}

pub fn init_llvm_destructors<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    if context.get_llvm_dtors().is_empty() {
        return;
    }

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_module: &Module = context.get_llvm_module();

    let dtor_type: StructType = llvm_context.struct_type(
        &[
            llvm_context.i32_type().into(),
            llvm_context.ptr_type(AddressSpace::default()).into(),
            llvm_context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let mut llvm_dtors: Vec<StructValue> = Vec::with_capacity(context.get_llvm_dtors().len());
    let mut last_counter: u32 = 0;

    for (ctor, counter) in context.get_llvm_dtors().iter() {
        if *counter > last_counter {
            let dtor_value: StructValue = dtor_type.const_named_struct(&[
                llvm_context
                    .i32_type()
                    .const_int((*counter).into(), false)
                    .into(),
                (*ctor).into(),
                llvm_context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into(),
            ]);

            llvm_dtors.push(dtor_value);
            last_counter = *counter;
        }
    }

    let actual_size: u32 = u32::try_from(llvm_dtors.len()).unwrap_or(u32::MAX - 1);

    let llvm_dtors_type: ArrayType = dtor_type.array_type(actual_size);
    let global: GlobalValue = llvm_module.add_global(llvm_dtors_type, None, "llvm.global_dtors");

    global.set_linkage(Linkage::Appending);
    global.set_initializer(&dtor_type.const_array(&llvm_dtors));
}
