use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::utils;
use crate::backends::classical::llvm::compiler::utils::SHORT_RANGE_OBFUSCATION;

use crate::frontends::classical::types::ast::metadata::constant::ConstantMetadata;
use crate::frontends::classical::types::ast::metadata::constant::LLVMConstantMetadata;
use crate::frontends::classical::types::ast::metadata::staticvar::LLVMStaticMetadata;
use crate::frontends::classical::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontends::classical::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;

use inkwell::{
    AddressSpace,
    module::{Linkage, Module},
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, GlobalValue, PointerValue},
};

pub fn local_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    metadata: ConstantMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let llvm_metadata: LLVMConstantMetadata = metadata.get_llvm_metadata();

    let name: String = format!(
        "{}.const.{}",
        utils::generate_random_string(SHORT_RANGE_OBFUSCATION),
        name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));
    global.set_linkage(Linkage::LinkerPrivate);

    global.set_unnamed_addr(true);
    global.set_constant(true);

    if llvm_metadata.thread_local {
        global.set_thread_local(true);
    }

    global.set_initializer(&llvm_value);

    global.as_pointer_value()
}

pub fn global_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
    metadata: ConstantMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let llvm_metadata: LLVMConstantMetadata = metadata.get_llvm_metadata();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate);
    }

    global.set_unnamed_addr(true);
    global.set_constant(true);

    if llvm_metadata.thread_local {
        global.set_thread_local(true);
    }

    global.set_initializer(&llvm_value);

    global.as_pointer_value()
}

pub fn local_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    metadata: StaticMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();

    let name: String = format!(
        "{}.static.{}",
        utils::generate_random_string(SHORT_RANGE_OBFUSCATION),
        name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    let alignment: u32 = target_data.get_preferred_alignment_of_global(&global);

    global.set_alignment(alignment);

    if llvm_metadata.constant {
        global.set_constant(true);
    }

    if llvm_metadata.unnamed_addr {
        global.set_unnamed_addr(true);
    }

    if llvm_metadata.thread_local {
        global.set_thread_local(true);
    }

    global.set_initializer(&llvm_value);
    global.set_linkage(Linkage::LinkerPrivate);

    global.as_pointer_value()
}

pub fn global_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
    metadata: StaticMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate);
    }

    if llvm_metadata.constant {
        global.set_constant(true);
    }

    if llvm_metadata.unnamed_addr {
        global.set_unnamed_addr(true);
    }

    if llvm_metadata.thread_local {
        global.set_thread_local(true);
    }

    global.set_initializer(&llvm_value);

    global.as_pointer_value()
}
