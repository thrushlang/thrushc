use thrushc_ast::Ast;
use thrushc_entities::AssemblerFunction;
use thrushc_llvm_attributes::LLVMAttribute;
use thrushc_llvm_attributes::LLVMAttributeComparator;
use thrushc_llvm_attributes::LLVMAttributes;
use thrushc_llvm_attributes::traits::LLVMAttributesExtensions;
use thrushc_llvm_callconventions::LLVMCallConvention;
use thrushc_span::Span;
use thrushc_typesystem::Type;
use thrushc_typesystem::traits::TypeIsExtensions;

use crate::abort;
use crate::attrbuilder::AttributeBuilder;
use crate::attrbuilder::LLVMAttributeApplicant;
use crate::block;
use crate::context::LLVMCodeGenContext;
use crate::obfuscation;
use crate::typegeneration;
use crate::types::LLVMFunction;

use inkwell::InlineAsmDialect;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, asm_fn: AssemblerFunction<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let name: &str = asm_fn.0;
    let ascii_name: &str = asm_fn.1;

    let assembler: String = asm_fn.2.to_string();
    let constraints: String = asm_fn.3.to_string();
    let return_type: &Type = asm_fn.4;
    let parameters: &[Ast] = asm_fn.5;
    let parameters_types: &[Type] = asm_fn.6;
    let attributes: LLVMAttributes = thrushc_llvm_attributes::into_llvm_attributes(asm_fn.7);

    let span: Span = asm_fn.8;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();
    let is_public: bool = attributes.has_public_attribute();

    let call_convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        LLVMCallConvention::Standard as u32
    };

    let syntax: InlineAsmDialect = match attributes.get_attr(LLVMAttributeComparator::AsmSyntax) {
        Some(LLVMAttribute::AsmSyntax(syntax), ..) => match syntax {
            "Intel" => InlineAsmDialect::Intel,
            "AT&T" => InlineAsmDialect::ATT,

            _ => InlineAsmDialect::Intel,
        },
        _ => InlineAsmDialect::Intel,
    };

    let llvm_function_name: String = if is_public {
        format!("__asm_fn_{}", ascii_name)
    } else {
        format!(
            "__asm_fn_{}_{}",
            obfuscation::generate_string(context, obfuscation::LONG_RANGE_OBFUSCATION),
            ascii_name
        )
    };

    let function_type: FunctionType =
        typegeneration::compile_as_function_type(context, return_type, parameters, false);

    let function_ptr: PointerValue = llvm_context.create_inline_asm(
        function_type,
        assembler,
        constraints,
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    let asm_function: FunctionValue =
        llvm_module.add_function(&llvm_function_name, function_type, None);

    AttributeBuilder::new(attributes, LLVMAttributeApplicant::Function(asm_function))
        .add_function_attributes(context);

    let last_block: BasicBlock = context.get_last_builder_block(span);
    let function_block: BasicBlock = block::append_block(context, asm_function);

    llvm_builder.position_at_end(function_block);

    let args: Vec<BasicMetadataValueEnum> = asm_function
        .get_param_iter()
        .map(|param| param.into())
        .collect();

    if let Ok(asm_fn_call) =
        llvm_builder.build_indirect_call(function_type, function_ptr, &args, "")
    {
        match (
            return_type.is_void_type(),
            asm_fn_call.try_as_basic_value().left(),
        ) {
            (false, Some(return_value)) => {
                llvm_builder
                    .build_return(Some(&return_value))
                    .map_err(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile assembly function!",
                            span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .ok();
            }
            _ => {
                llvm_builder
                    .build_return(None)
                    .map_err(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile assembly function!",
                            span,
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .ok();
            }
        }
    } else {
        abort::abort_codegen(
            context,
            "Failed to compile indirect call for assembly function!",
            span,
            std::path::PathBuf::from(file!()),
            line!(),
        );
    }

    llvm_builder.position_at_end(last_block);

    let proto: LLVMFunction = (
        asm_function,
        return_type,
        parameters_types,
        call_convention,
        span,
    );

    context.new_function(name, proto);
}
