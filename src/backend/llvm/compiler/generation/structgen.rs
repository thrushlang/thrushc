use std::fmt::Display;
use std::sync::Arc;

use crate::backend::llvm::compiler::anchors::PointerAnchor;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::{typegen, valuegen};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::parser::stmts::types::Constructor;
use crate::frontend::typesystem::traits::TypeStructExtensions;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_struct_with_anchor(context, args, kind, cast, anchor)
        } else {
            self::compile_struct_without_anchor(context, args, kind, cast)
        }
    } else {
        self::compile_struct_without_anchor(context, args, kind, cast)
    }
}

fn compile_struct_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    kind: &Type,
    cast: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let struct_type: &Type = cast.unwrap_or(kind);

    let struct_llvm_type: BasicTypeEnum =
        typegen::generate_type(context.get_llvm_context(), struct_type);

    let struct_ptr: PointerValue = anchor.get_pointer();

    context.set_pointer_anchor(PointerAnchor::new(struct_ptr, true));

    let struct_fields_types: &[Arc<Type>] = struct_type.get_struct_fields();

    let fields: Vec<BasicValueEnum> = args
        .iter()
        .zip(struct_fields_types)
        .map(|((_, field, _, _), kind)| valuegen::compile(context, field, Some(kind)))
        .collect();

    for (idx, field) in fields.iter().enumerate() {
        match context.get_llvm_builder().build_struct_gep(
            struct_llvm_type,
            struct_ptr,
            idx as u32,
            "",
        ) {
            Ok(ptr) => {
                memory::store_anon(context, ptr, *field);
            }
            Err(err) => {
                self::codegen_abort(err);
                self::compile_null_ptr(context);
            }
        }
    }

    self::compile_null_ptr(context)
}

pub fn compile_struct_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let struct_type: &Type = cast.unwrap_or(kind);

    let struct_llvm_type: BasicTypeEnum =
        typegen::generate_type(context.get_llvm_context(), struct_type);

    let struct_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, struct_type);

    let struct_fields_types: &[Arc<Type>] = struct_type.get_struct_fields();

    let fields: Vec<BasicValueEnum> = args
        .iter()
        .zip(struct_fields_types)
        .map(|((_, field, _, _), kind)| valuegen::compile(context, field, Some(kind)))
        .collect();

    for (idx, field) in fields.iter().enumerate() {
        match context.get_llvm_builder().build_struct_gep(
            struct_llvm_type,
            struct_ptr,
            idx as u32,
            "",
        ) {
            Ok(ptr) => {
                memory::store_anon(context, ptr, *field);
            }
            Err(err) => {
                self::codegen_abort(err);
                self::compile_null_ptr(context);
            }
        }
    }

    memory::load_anon(context, struct_ptr, struct_type)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
