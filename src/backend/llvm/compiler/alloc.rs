use std::fmt::Display;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, GlobalValue, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        typegen,
        utils::{self, SHORT_RANGE_OBFUSCATION},
    },
    core::console::logging::{self, LoggingType},
    frontend::{
        types::{
            ast::metadata::staticvar::{LLVMStaticMetadata, StaticMetadata},
            parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn alloc<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    kind: &Type,
    attributes: &ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let formatted_name: String = format!(
        "{}.local.{}",
        utils::generate_random_string(SHORT_RANGE_OBFUSCATION),
        ascii_name
    );

    match (attributes.has_heap_attr(), attributes.has_stack_attr()) {
        (true, _) => self::try_alloc_heap(context, llvm_type, &formatted_name, kind),
        (_, true) => self::try_alloc_stack(context, llvm_type, &formatted_name, kind),
        _ => self::try_alloc_stack(context, llvm_type, &formatted_name, kind),
    }
}

fn try_alloc_heap<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    kind: &Type,
) -> PointerValue<'ctx> {
    match context
        .get_llvm_builder()
        .build_malloc(llvm_type, ascii_name)
    {
        Ok(ptr) => ptr,
        Err(_) => {
            self::codegen_abort(format!(
                "Failed to allocate heap memory for type '{}'.",
                kind
            ));

            unreachable!()
        }
    }
}

fn try_alloc_stack<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    kind: &Type,
) -> PointerValue<'ctx> {
    match context
        .get_llvm_builder()
        .build_alloca(llvm_type, ascii_name)
    {
        Ok(ptr) => ptr,
        Err(_) => {
            self::codegen_abort(format!(
                "Failed to allocate stack memory for type '{}'.",
                kind
            ));

            unreachable!()
        }
    }
}

pub fn local_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let name: String = format!(
        "{}.const.{}",
        utils::generate_random_string(SHORT_RANGE_OBFUSCATION),
        ascii_name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));
    global.set_linkage(Linkage::LinkerPrivate);

    global.set_unnamed_addr(true);
    global.set_initializer(&llvm_value);
    global.set_constant(true);

    global.as_pointer_value()
}

pub fn global_constant<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let target_data: &TargetData = context.get_target_data();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), ascii_name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate);
    }

    global.set_unnamed_addr(true);
    global.set_constant(true);

    global.set_initializer(&llvm_value);

    global.as_pointer_value()
}

pub fn local_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    metadata: StaticMetadata,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();
    let target_data: &TargetData = context.get_target_data();

    let name: String = format!(
        "{}.static.{}",
        utils::generate_random_string(SHORT_RANGE_OBFUSCATION),
        ascii_name
    );

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), &name);

    let alignment: u32 = target_data.get_preferred_alignment_of_global(&global);

    global.set_alignment(alignment);

    if llvm_metadata.can_constant {
        global.set_constant(true);
    }

    if llvm_metadata.can_unnamed_addr {
        global.set_unnamed_addr(true);
    }

    global.set_initializer(&llvm_value);
    global.set_linkage(Linkage::LinkerPrivate);

    global.as_pointer_value()
}

pub fn global_static<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    metadata: StaticMetadata,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_metadata: LLVMStaticMetadata = metadata.get_llvm_metadata();
    let target_data: &TargetData = context.get_target_data();

    let global: GlobalValue =
        llvm_module.add_global(llvm_type, Some(AddressSpace::default()), ascii_name);

    global.set_alignment(target_data.get_preferred_alignment_of_global(&global));

    if llvm_metadata.can_constant {
        global.set_constant(true);
    }

    if llvm_metadata.can_unnamed_addr {
        global.set_unnamed_addr(true);
    }

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate);
    }

    global.set_initializer(&llvm_value);

    global.as_pointer_value()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
