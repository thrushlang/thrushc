use std::iter;

use inkwell::values::StructValue;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::GlobalValue,
};
use rand::Rng;
use rand::rngs::ThreadRng;

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
