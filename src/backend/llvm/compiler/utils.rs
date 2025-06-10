use inkwell::values::PointerValue;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::GlobalValue,
};

pub fn build_str_constant<'ctx>(
    module: &Module<'ctx>,
    context: &'ctx Context,
    bytes: &'ctx [u8],
) -> PointerValue<'ctx> {
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

    global.as_pointer_value()
}

pub fn generate_assembler_function_name(function_name: &str) -> String {
    format!("__assembler_fn_{}", function_name)
}
