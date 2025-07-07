use inkwell::{context::Context, values::IntValue};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, intgen, valuegen},
    frontend::{
        types::ast::Ast,
        typesystem::{traits::TypeMutableExtensions, types::Type},
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [Ast],
    expr_type: &'ctx Type,
) -> Vec<IntValue<'ctx>> {
    let llvm_context: &Context = context.get_llvm_context();

    indexes
        .iter()
        .flat_map(|index| {
            if expr_type.is_fixed_array_type() || expr_type.is_mut_fixed_array_type() {
                let base: IntValue = intgen::integer(llvm_context, &Type::U32, 0, false);

                let depth: IntValue =
                    valuegen::compile(context, index, Some(&Type::U32)).into_int_value();

                vec![base, depth]
            } else {
                let depth: IntValue =
                    valuegen::compile(context, index, Some(&Type::U64)).into_int_value();

                vec![depth]
            }
        })
        .collect()
}
