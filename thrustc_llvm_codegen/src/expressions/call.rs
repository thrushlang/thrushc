use thrustc_ast::Ast;
use thrustc_typesystem::Type;
use thrustc_typesystem::traits::TypeIsExtensions;

use crate::abort;
use crate::cast;
use crate::codegen;
use crate::context::LLVMCodeGenContext;
use crate::types::LLVMFunction;

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    args: &'ctx [Ast],
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let function: LLVMFunction = context.get_table().get_function(name);

    let (llvm_function, function_arg_types, function_convention, span) =
        (function.0, function.2, function.3, function.4);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .enumerate()
        .map(|(i, expr)| {
            let cast: Option<&Type> = function_arg_types.get(i);
            codegen::compile(context, expr, cast).into()
        })
        .collect();

    let ret_value: BasicValueEnum = match llvm_builder.build_call(llvm_function, &compiled_args, "")
    {
        Ok(call) => {
            call.set_call_convention(function_convention);

            if !kind.is_void_type() {
                call.try_as_basic_value().left().unwrap_or_else(|| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile function call!",
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
            "Failed to compile function call!",
            span,
            std::path::PathBuf::from(file!()),
            line!(),
        ),
    };

    cast::try_cast(context, cast, kind, ret_value, span)
}
