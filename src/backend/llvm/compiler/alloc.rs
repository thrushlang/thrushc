use std::fmt::Display;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    core::console::logging::{self, LoggingType},
    frontend::types::{
        lexer::ThrushType,
        parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
    },
};

pub fn alloc<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    kind: &ThrushType,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let target_data: &TargetData = context.get_target_data();
    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    match (
        attributes.has_heap_attr(),
        attributes.has_stack_attr(),
        kind.is_probably_heap_allocated(llvm_context, target_data),
    ) {
        (true, _, _) => self::try_alloc_heap(context, llvm_type, name, kind),
        (false, true, _) => self::try_alloc_stack(context, llvm_type, name, kind),
        (false, false, true) => self::try_alloc_heap(context, llvm_type, name, kind),
        _ => self::try_alloc_stack(context, llvm_type, name, kind),
    }
}

fn try_alloc_heap<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    name: &str,
    kind: &ThrushType,
) -> PointerValue<'ctx> {
    match context.get_llvm_builder().build_malloc(llvm_type, name) {
        Ok(ptr) => ptr,
        Err(_) => {
            self::codegen_abort(format!(
                "Failed to allocate heap memory for type '{}'",
                kind
            ));

            unreachable!()
        }
    }
}

fn try_alloc_stack<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    name: &str,
    kind: &ThrushType,
) -> PointerValue<'ctx> {
    match context.get_llvm_builder().build_alloca(llvm_type, name) {
        Ok(ptr) => ptr,
        Err(_) => {
            self::codegen_abort(format!(
                "Failed to allocate stack memory for type '{}'",
                kind
            ));

            unreachable!()
        }
    }
}

pub fn constant<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let global = module.add_global(llvm_type, Some(AddressSpace::default()), name);
    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate);
    }
    global.set_initializer(&llvm_value);
    global.set_constant(true);
    global.as_pointer_value()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::Bug, &format!("CODE GENERATION: '{}'", message));
}
