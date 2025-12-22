use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::obfuscation;
use crate::core::diagnostic::span::Span;

use std::path::PathBuf;

use inkwell::values::PointerValue;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::ArrayType;
use inkwell::values::GlobalValue;

pub fn compile_str_constant<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    bytes: &'ctx [u8],
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let parsed_size: u32 = u32::try_from(bytes.len()).unwrap_or_else(|_| {
        abort::abort_codegen(
            context,
            "Failed to extract the from struct!",
            span,
            PathBuf::from(file!()),
            line!(),
        )
    });

    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        parsed_size + 1
    } else {
        parsed_size
    };

    let cstr_type: ArrayType = llvm_context.i8_type().array_type(fixed_cstr_size);

    let cstr_name: String = format!(
        "cstr.constant{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION)
    );

    let cstr: GlobalValue =
        llvm_module.add_global(cstr_type, Some(AddressSpace::default()), &cstr_name);

    cstr.set_alignment(
        context
            .get_target_data()
            .get_preferred_alignment_of_global(&cstr),
    );

    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_constant(true);

    cstr.as_pointer_value()
}

pub fn compile_str<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    bytes: &[u8],
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let parsed_size: u32 = u32::try_from(bytes.len()).unwrap_or_else(|_| {
        abort::abort_codegen(
            context,
            "Failed to extract the from struct!",
            span,
            PathBuf::from(file!()),
            line!(),
        )
    });

    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        parsed_size + 1
    } else {
        parsed_size
    };

    let cstr_name: String = format!(
        "cstr{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION)
    );

    let cstr_type: ArrayType = llvm_context.i8_type().array_type(fixed_cstr_size);
    let cstr: GlobalValue =
        llvm_module.add_global(cstr_type, Some(AddressSpace::default()), &cstr_name);

    cstr.set_alignment(
        context
            .get_target_data()
            .get_preferred_alignment_of_global(&cstr),
    );
    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_unnamed_addr(true);
    cstr.set_constant(true);

    cstr.as_pointer_value()
}
