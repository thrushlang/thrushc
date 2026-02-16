use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::FunctionType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use inkwell::{AddressSpace, InlineAsmDialect};

use thrustc_ast::Ast;
use thrustc_llvm_attributes::traits::LLVMAttributesExtensions;
use thrustc_llvm_attributes::{LLVMAttribute, LLVMAttributeComparator, LLVMAttributes};
use thrustc_span::Span;
use thrustc_typesystem::Type;
use thrustc_typesystem::traits::TypeIsExtensions;

use crate::abort;
use crate::context::LLVMCodeGenContext;
use crate::{codegen, typegeneration};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    assembler: &str,
    constraints: &str,
    args: &'ctx [Ast],
    kind: &Type,
    attributes: LLVMAttributes,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_type: FunctionType =
        typegeneration::compile_as_function_type(context, kind, args, false);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .map(|arg| codegen::compile(context, arg, None).into())
        .collect();

    let syntax: InlineAsmDialect = match attributes.get_attr(LLVMAttributeComparator::AsmSyntax) {
        Some(LLVMAttribute::AsmSyntax(syntax), ..) => match syntax {
            "Intel" => InlineAsmDialect::Intel,
            "AT&T" => InlineAsmDialect::ATT,

            _ => InlineAsmDialect::Intel,
        },
        _ => InlineAsmDialect::Intel,
    };

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();

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
            abort::abort_codegen(
                context,
                "Failed to compile a inlineassembler value!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            );
        }),

        Ok(_) => self::compile_null_ptr(context),

        Err(_) => {
            abort::abort_codegen(
                context,
                "Failed to compile a inlineassembler value!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            );
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
