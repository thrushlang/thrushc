use std::path::PathBuf;

use inkwell::AddressSpace;
use inkwell::{context::Context, values::BasicValueEnum};

use crate::back_end::llvm_codegen::builtins;
use crate::back_end::llvm_codegen::builtins::LLVMBuiltin;
use crate::back_end::llvm_codegen::constgen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::generation::expressions::unary;
use crate::back_end::llvm_codegen::generation::float;
use crate::back_end::llvm_codegen::generation::integer;
use crate::back_end::llvm_codegen::{abort, refptr};
use crate::back_end::llvm_codegen::{binaryop, generation};

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstLLVMGetType};
use crate::front_end::typesystem::traits::TypeStructExtensions;
use crate::front_end::typesystem::types::Type;

pub fn compile<'ctx>(
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
                float::generate_const(context, kind, *value, *signed, *span).into();

            generation::cast::const_numeric_cast(context, float, cast_type, *signed)
        }

        Ast::Integer {
            value,
            kind,
            signed,
            span,
            ..
        } => {
            let integer: BasicValueEnum =
                integer::generate_const(context, kind, *value, *signed, *span).into();

            generation::cast::const_numeric_cast(context, integer, cast_type, *signed)
        }

        // Boolean true/false cases
        Ast::Boolean { value, .. } => context
            .get_llvm_context()
            .bool_type()
            .const_int(*value, false)
            .into(),

        // Fixed-size array
        Ast::FixedArray { items, span, .. } => {
            generation::expressions::farray::compile_const(context, items, cast_type, *span)
        }

        // String literal compilation
        Ast::Str { bytes, span, .. } => {
            generation::expressions::string::compile_str_constant(context, bytes, *span).into()
        }

        // Struct constructor handling
        Ast::Constructor { args, kind, .. } => {
            let fields: Vec<&Ast> = args.iter().map(|raw_arg| &raw_arg.1).collect();

            let llvm_context: &Context = context.get_llvm_context();

            let struct_fields_types: &[Type] = kind.get_struct_fields();

            let fields: Vec<BasicValueEnum> = fields
                .iter()
                .zip(struct_fields_types)
                .map(|(field, kind)| constgen::compile(context, field, kind))
                .collect();

            llvm_context.const_struct(&fields, false).into()
        }

        // Type cast_typeing operations
        Ast::As { from, cast, .. } => {
            let lhs_type: &Type = from.llvm_get_type(context);
            let lhs: BasicValueEnum = constgen::compile(context, from, lhs_type);

            generation::cast::try_cast_const(context, lhs, lhs_type, cast)
        }

        // Variable reference resolution
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_value(context),

        // Grouped expression compilation
        Ast::Group { expression, .. } => self::compile(context, expression, cast_type),

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
                    cast_type,
                );
            }

            if binaryop_type.is_bool_type() {
                return binaryop::boolean::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast_type,
                );
            }

            if binaryop_type.is_float_type() {
                return binaryop::float::compile_const(
                    context,
                    (left, operator, right, *span),
                    cast_type,
                );
            }

            abort::abort_codegen(
                context,
                "Failed to compile the binary operation!",
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
        } => unary::compile_const(context, (operator, kind, expression), cast_type),

        // Direct Reference
        Ast::DirectRef { expr, .. } => refptr::compile(context, expr, None),

        // Builtins
        Ast::Builtin { builtin, .. } => {
            let llvm_builtin: LLVMBuiltin<'_> = builtin.to_llvm_builtin();
            builtins::compile(context, llvm_builtin, Some(cast_type))
        }

        // Enum Value Access
        Ast::EnumValue { value, .. } => self::compile(context, value, cast_type),

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
