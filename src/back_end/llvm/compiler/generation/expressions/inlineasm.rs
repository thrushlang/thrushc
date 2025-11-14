use std::fmt::Display;

use crate::back_end::llvm::compiler::attributes::{LLVMAttribute, LLVMAttributeComparator};
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::{codegen, typegen};
use crate::back_end::llvm::types::repr::LLVMAttributes;
use crate::back_end::llvm::types::traits::{AssemblerFunctionExtensions, LLVMAttributesExtensions};

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::FunctionType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use inkwell::{AddressSpace, InlineAsmDialect};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    assembler: &str,
    constraints: &str,
    args: &'ctx [Ast],
    kind: &Type,
    attributes: LLVMAttributes,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_type: FunctionType = typegen::generate_fn_type(context, kind, args, false);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .map(|arg| codegen::compile(context, arg, None).into())
        .collect();

    let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();

    if let Some(LLVMAttribute::AsmSyntax(new_syntax, ..)) =
        attributes.get_attr(LLVMAttributeComparator::AsmSyntax)
    {
        syntax = str::to_inline_assembler_dialect(new_syntax);
    }

    let fn_inline_assembler: PointerValue = llvm_context.create_inline_asm(
        asm_function_type,
        assembler.to_string(),
        constraints.to_string(),
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    match llvm_builder.build_indirect_call(
        asm_function_type,
        fn_inline_assembler,
        &compiled_args,
        "",
    ) {
        Ok(call) if !kind.is_void_type() => call.try_as_basic_value().left().unwrap_or_else(|| {
            self::codegen_abort("Inline assembler returned no value.");
        }),

        Ok(_) => self::compile_null_ptr(context),

        Err(_) => {
            self::codegen_abort("Failed to build inline assembler.");
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
