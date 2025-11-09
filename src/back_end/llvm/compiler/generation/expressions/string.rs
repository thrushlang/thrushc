use inkwell::targets::TargetData;
use inkwell::values::PointerValue;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::GlobalValue,
};

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::obfuscation;

pub fn compile_str_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    bytes: &'ctx [u8],
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let target_data: &TargetData = context.get_target_data();

    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        bytes.len() as u32 + 1
    } else {
        bytes.len() as u32
    };

    let cstr_type: ArrayType = llvm_context.i8_type().array_type(fixed_cstr_size);

    let cstr_name: String = format!(
        "cstr.constant{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION)
    );

    let cstr: GlobalValue =
        llvm_module.add_global(cstr_type, Some(AddressSpace::default()), &cstr_name);

    cstr.set_alignment(target_data.get_preferred_alignment_of_global(&cstr));
    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_constant(true);

    cstr.as_pointer_value()
}

pub fn compile_str<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    bytes: &[u8],
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let target_data: &TargetData = context.get_target_data();

    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        bytes.len() as u32 + 1
    } else {
        bytes.len() as u32
    };

    let cstr_name: String = format!(
        "cstr{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION)
    );

    let cstr_type: ArrayType = llvm_context.i8_type().array_type(fixed_cstr_size);
    let cstr: GlobalValue =
        llvm_module.add_global(cstr_type, Some(AddressSpace::default()), &cstr_name);

    cstr.set_alignment(target_data.get_preferred_alignment_of_global(&cstr));
    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_unnamed_addr(true);
    cstr.set_constant(true);

    cstr.as_pointer_value()
}
