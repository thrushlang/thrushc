use std::path::PathBuf;
use std::sync::Arc;

use inkwell::{context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::abort;
use crate::backends::classical::llvm::compiler::binaryop;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::generation::expressions::unary;
use crate::backends::classical::llvm::compiler::generation::float;
use crate::backends::classical::llvm::compiler::generation::int;
use crate::backends::classical::llvm::compiler::typegen;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::traits::TypeStructExtensions;
use crate::frontends::classical::typesystem::types::Type;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ast: &'ctx Ast,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    match ast {
        // Handle integer literals
        Ast::Integer {
            value,
            kind,
            signed,
            ..
        } => {
            let int: BasicValueEnum =
                int::generate(context.get_llvm_context(), kind, *value, *signed).into();

            let cast: BasicTypeEnum = typegen::generate_type(context.get_llvm_context(), cast);

            compiler::generation::cast::numeric_cast(int, cast, *signed)
        }

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
            ..
        } => {
            let float: BasicValueEnum =
                float::generate(context.get_llvm_context(), kind, *value, *signed).into();

            let cast: BasicTypeEnum = typegen::generate_type(context.get_llvm_context(), cast);

            compiler::generation::cast::numeric_cast(float, cast, *signed)
        }

        // Boolean true/false cases
        Ast::Boolean { value, .. } => context
            .get_llvm_context()
            .bool_type()
            .const_int(*value, false)
            .into(),

        // Fixed-size array
        Ast::FixedArray { items, .. } => {
            compiler::generation::expressions::farray::compile_const(context, items, cast)
        }

        // String literal compilation
        Ast::Str { bytes, .. } => {
            compiler::generation::expressions::string::compile_str(context, bytes).into()
        }

        // Struct constructor handling
        Ast::Constructor { args, kind, .. } => {
            let fields: Vec<&Ast> = args.iter().map(|raw_arg| &raw_arg.1).collect();

            let llvm_context: &Context = context.get_llvm_context();

            let struct_fields_types: &[Arc<Type>] = kind.get_struct_fields();

            let fields: Vec<BasicValueEnum> = fields
                .iter()
                .zip(struct_fields_types)
                .map(|(field, kind)| constgen::compile(context, field, kind))
                .collect();

            llvm_context.const_struct(&fields, false).into()
        }

        // Type casting operations
        Ast::As { from, cast, .. } => {
            let lhs_type: &Type = from.get_type_unwrapped();
            let lhs: BasicValueEnum = constgen::compile(context, from, lhs_type);

            compiler::generation::cast::try_cast_const(context, lhs, lhs_type, cast)
        }

        // Variable reference resolution
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_value(),

        // Grouped expression compilation
        Ast::Group { expression, .. } => self::compile(context, expression, cast),

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
                return binaryop::integer::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast,
                );
            }

            if binaryop_type.is_bool_type() {
                return binaryop::boolean::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast,
                );
            }

            if binaryop_type.is_float_type() {
                return binaryop::float::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast,
                );
            }

            if binaryop_type.is_ptr_type() {
                return binaryop::pointer::compile_const(context, (left, operator, right, *span));
            }

            abort::abort_codegen(
                context,
                "Can't be compiled!.",
                *span,
                PathBuf::from(file!()),
                line!(),
            );
        }

        // Unary operation dispatch
        Ast::UnaryOp {
            operator,
            expression,
            kind,
            ..
        } => unary::compile_const(context, (operator, kind, expression), cast),

        // Fallback for unsupported AST nodes
        what => abort::abort_codegen(
            context,
            "Unknown expression or statement!",
            what.get_span(),
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
