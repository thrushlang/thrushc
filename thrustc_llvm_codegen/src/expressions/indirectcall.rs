use thrustc_ast::Ast;
use thrustc_span::Span;
use thrustc_typesystem::Type;
use thrustc_typesystem::traits::TypeIsExtensions;

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::types::FunctionType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};

use crate::context::LLVMCodeGenContext;
use crate::{abort, cast, codegen, typegeneration};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    pointer: &'ctx Ast,
    args: &'ctx [Ast],
    function_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder<'_> = context.get_llvm_builder();
    let function_pointer: PointerValue<'_> =
        codegen::compile_as_ptr(context, pointer, cast_type).into_pointer_value();

    if let Type::Fn(parameters, kind, modificator, ..) = function_type {
        let need_ignore: bool = modificator.llvm().has_ignore();

        let function_type: FunctionType<'_> =
            typegeneration::compile_from_function_type_to_function_type(
                context,
                kind,
                parameters,
                need_ignore,
            );

        let compiled_args: Vec<BasicMetadataValueEnum> = args
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let cast_type = parameters.get(i);
                codegen::compile(context, expr, cast_type).into()
            })
            .collect();

        let fn_value: BasicValueEnum<'_> = match llvm_builder.build_indirect_call(
            function_type,
            function_pointer,
            &compiled_args,
            "",
        ) {
            Ok(call) => {
                if !kind.is_void_type() {
                    call.try_as_basic_value().left().unwrap_or_else(|| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile indirect function call!",
                            span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        )
                    })
                } else {
                    context
                        .get_llvm_context()
                        .ptr_type(AddressSpace::default())
                        .const_null()
                        .into()
                }
            }
            Err(_) => abort::abort_codegen(
                context,
                "Failed to compile indirect function call!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            ),
        };

        cast::try_cast(context, cast_type, kind, fn_value, span)
    } else {
        abort::abort_codegen(
            context,
            "Failed to compile indirect function call!",
            span,
            std::path::PathBuf::from(file!()),
            line!(),
        )
    }
}
