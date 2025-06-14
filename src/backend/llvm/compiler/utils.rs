use inkwell::types::StructType;
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
    let fixed_cstr_size: u32 = if !bytes.is_empty() {
        bytes.len() as u32 + 1
    } else {
        bytes.len() as u32
    };

    let cstr_type: ArrayType = context.i8_type().array_type(fixed_cstr_size);
    let cstr: GlobalValue = module.add_global(cstr_type, Some(AddressSpace::default()), "");

    cstr.set_linkage(Linkage::LinkerPrivate);
    cstr.set_initializer(&context.const_string(bytes, true));
    cstr.set_unnamed_addr(true);
    cstr.set_constant(true);

    let str_type: StructType = context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    );
    let str: GlobalValue = module.add_global(str_type, Some(AddressSpace::default()), "");

    str.set_linkage(Linkage::LinkerPrivate);
    str.set_initializer(
        &context.const_struct(
            &[
                cstr.as_pointer_value().into(),
                context
                    .i64_type()
                    .const_int(fixed_cstr_size as u64, false)
                    .into(),
            ],
            false,
        ),
    );

    str.set_constant(true);
    str.as_pointer_value()
}

pub fn generate_assembler_function_name(function_name: &str) -> String {
    format!("__assembler_fn_{}", function_name)
}
