use std::fmt::Display;

use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::{self, ptr};
use crate::backends::classical::llvm::compiler::{codegen, typegen};

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::types::FunctionType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,

    pointer: &'ctx Ast,
    args: &'ctx [Ast],
    function_type: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let function_ptr: PointerValue = ptr::compile(context, pointer, cast).into_pointer_value();

    match function_type {
        Type::Fn(parameters, kind) => {
            let function_type: FunctionType =
                typegen::function_type_from_type(context, kind, parameters, false);

            let compiled_args: Vec<BasicMetadataValueEnum> = args
                .iter()
                .enumerate()
                .map(|(i, expr)| {
                    let cast: Option<&Type> = parameters.get(i);
                    codegen::compile_expr(context, expr, cast).into()
                })
                .collect();

            let fn_value: BasicValueEnum = match llvm_builder.build_indirect_call(
                function_type,
                function_ptr,
                &compiled_args,
                "",
            ) {
                Ok(call) => {
                    if !kind.is_void_type() {
                        call.try_as_basic_value().left().unwrap_or_else(|| {
                            self::codegen_abort(
                                "Function indirect reference call not returned a value.",
                            );
                        })
                    } else {
                        context
                            .get_llvm_context()
                            .ptr_type(AddressSpace::default())
                            .const_null()
                            .into()
                    }
                }
                Err(_) => {
                    self::codegen_abort("Failed to generate indirect call.");
                }
            };

            compiler::generation::cast::try_cast(context, cast, kind, fn_value).unwrap_or(fn_value)
        }
        _ => {
            self::codegen_abort("Expected function reference.");
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
