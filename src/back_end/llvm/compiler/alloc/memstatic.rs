#![allow(clippy::too_many_arguments)]

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::obfuscation;

use crate::front_end::types::ast::metadata::constant::{ConstantMetadata, LLVMConstantMetadata};
use crate::front_end::types::ast::metadata::staticvar::{LLVMStaticMetadata, StaticMetadata};
use crate::front_end::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;

use inkwell::ThreadLocalMode;
use inkwell::module::Module;
use inkwell::{
    AddressSpace,
    module::Linkage,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, GlobalValue, PointerValue},
};

fn set_global_common<'ctx>(
    global: &GlobalValue<'ctx>,
    constant: bool,
    unnamed_addr: bool,
    thread_local: bool,
    thread_mode: Option<ThreadLocalMode>,
    initializer: Option<&BasicValueEnum<'ctx>>,
    alignment: Option<u32>,
    linkage: Option<Linkage>,
) {
    if let Some(align) = alignment {
        global.set_alignment(align);
    }
    if let Some(link) = linkage {
        global.set_linkage(link);
    }
    if constant {
        global.set_constant(true);
    }
    if unnamed_addr {
        global.set_unnamed_addr(true);
    }
    if thread_local {
        global.set_thread_local(true);
    }
    if let Some(mode) = thread_mode {
        global.set_thread_local_mode(Some(mode));
    }
    if let Some(init) = initializer {
        global.set_initializer(init);
    }
}

pub fn local_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    value: BasicValueEnum<'ctx>,
    metadata: ConstantMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();
    let llvm_metadata: LLVMConstantMetadata = metadata.get_llvm_metadata();

    let name: String = format!(
        "local.{}.const{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION),
        name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    self::set_global_common(
        &global,
        true,
        true,
        llvm_metadata.thread_local,
        None,
        Some(&value),
        Some(target_data.get_preferred_alignment_of_global(&global)),
        Some(Linkage::LinkerPrivate),
    );

    global.as_pointer_value()
}

pub fn global_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
    metadata: ConstantMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();
    let llvm_metadata: LLVMConstantMetadata = metadata.get_llvm_metadata();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), name);

    self::set_global_common(
        &global,
        true,
        true,
        llvm_metadata.thread_local,
        None,
        Some(&value),
        Some(target_data.get_preferred_alignment_of_global(&global)),
        if !attributes.has_public_attribute() {
            Some(Linkage::LinkerPrivate)
        } else {
            None
        },
    );

    global.as_pointer_value()
}

pub fn local_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    value: Option<BasicValueEnum<'ctx>>,
    metadata: StaticMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();
    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();

    let name: String = format!(
        "local.{}.static{}",
        obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION),
        name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    self::set_global_common(
        &global,
        llvm_metadata.constant,
        llvm_metadata.unnamed_addr,
        llvm_metadata.thread_local,
        llvm_metadata.thread_mode,
        value.as_ref(),
        Some(target_data.get_preferred_alignment_of_global(&global)),
        Some(Linkage::LinkerPrivate),
    );

    global.as_pointer_value()
}

pub fn global_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    value: Option<BasicValueEnum<'ctx>>,
    attributes: &'ctx ThrushAttributes<'ctx>,
    metadata: StaticMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();
    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), name);

    self::set_global_common(
        &global,
        llvm_metadata.constant,
        llvm_metadata.unnamed_addr,
        llvm_metadata.thread_local,
        llvm_metadata.thread_mode,
        value.as_ref(),
        Some(target_data.get_preferred_alignment_of_global(&global)),
        if !attributes.has_public_attribute() {
            Some(Linkage::LinkerPrivate)
        } else {
            None
        },
    );

    global.as_pointer_value()
}
