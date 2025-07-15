use inkwell::targets::TargetData;
use inkwell::types::StructType;
use inkwell::values::{PointerValue, StructValue};

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::GlobalValue,
};

use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::utils;

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

    let cstr_name: String = format!("cstr.constant.{}", utils::generate_random_string());

    let cstr: GlobalValue =
        llvm_module.add_global(cstr_type, Some(AddressSpace::default()), &cstr_name);

    cstr.set_alignment(target_data.get_preferred_alignment_of_global(&cstr));
    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_constant(true);

    let str_type: StructType = llvm_context.struct_type(
        &[
            llvm_context.ptr_type(AddressSpace::default()).into(),
            llvm_context.i32_type().into(),
        ],
        false,
    );

    let str_name: String = format!("str.constant.{}", utils::generate_random_string());

    let str: GlobalValue =
        llvm_module.add_global(str_type, Some(AddressSpace::default()), &str_name);

    str.set_alignment(target_data.get_preferred_alignment_of_global(&str));

    str.set_linkage(Linkage::LinkerPrivate);

    str.set_initializer(
        &llvm_context.const_struct(
            &[
                cstr.as_pointer_value().into(),
                llvm_context
                    .i32_type()
                    .const_int(fixed_cstr_size as u64, false)
                    .into(),
            ],
            false,
        ),
    );

    str.set_constant(true);
    str.as_pointer_value()
}

pub fn compile_str<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    bytes: &[u8],
) -> StructValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        bytes.len() as u32 + 1
    } else {
        bytes.len() as u32
    };

    let cstr_type: ArrayType = llvm_context.i8_type().array_type(fixed_cstr_size);
    let cstr: GlobalValue = llvm_module.add_global(cstr_type, Some(AddressSpace::default()), "");

    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&llvm_context.const_string(bytes, true));
    cstr.set_unnamed_addr(true);
    cstr.set_constant(true);

    llvm_context.const_struct(
        &[
            cstr.as_pointer_value().into(),
            llvm_context
                .i32_type()
                .const_int(fixed_cstr_size as u64, false)
                .into(),
        ],
        false,
    )
}
