use std::iter;

use crate::middle::types::frontend::lexer::types::ThrushType;

use super::context::LLVMCodeGenContext;
use super::typegen;

use inkwell::values::StructValue;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::{BasicValueEnum, GlobalValue},
};
use rand::Rng;
use rand::rngs::ThreadRng;

pub fn integer_autocast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &ThrushType,
    from_type: &ThrushType,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::thrush_integer_to_llvm_type(llvm_context, target_type),
                true,
                "",
            )
            .unwrap()
            .into(),
    )
}

pub fn float_autocast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &ThrushType,
    from_type: &ThrushType,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_float_cast(
                from.into_float_value(),
                typegen::type_float_to_llvm_float_type(llvm_context, target_type),
                "",
            )
            .unwrap()
            .into(),
    )
}

pub fn build_str_constant<'ctx>(
    module: &Module<'ctx>,
    context: &'ctx Context,
    bytes: &'ctx [u8],
) -> StructValue<'ctx> {
    let fixed_str_size: u32 = if !bytes.is_empty() {
        bytes.len() as u32 + 1
    } else {
        bytes.len() as u32
    };

    let kind: ArrayType = context.i8_type().array_type(fixed_str_size);
    let global: GlobalValue = module.add_global(kind, Some(AddressSpace::default()), "");

    global.set_linkage(Linkage::LinkerPrivate);
    global.set_initializer(&context.const_string(bytes, true));
    global.set_constant(true);

    context.const_struct(
        &[
            global.as_pointer_value().into(),
            context
                .i64_type()
                .const_int(fixed_str_size as u64, false)
                .into(),
        ],
        false,
    )
}

#[inline]
pub fn generate_random_function_name(prefix: &str, length: usize) -> String {
    format!("{}_{}", prefix, generate_random_suffix(length))
}

#[inline]
pub fn generate_random_range(max: usize) -> usize {
    rand::rng().random_range(0..max)
}

fn generate_random_suffix(length: usize) -> String {
    let letters: String = String::from("abcdefghijklmnopqrstuvwxyz0123456789");
    let mut rng: ThreadRng = rand::rng();

    iter::repeat(())
        .map(|_| rng.random_range(0..letters.len()))
        .map(|i| letters.chars().nth(i).unwrap())
        .take(length)
        .collect()
}
