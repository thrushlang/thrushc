use inkwell::{context::Context, values::IntValue};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, generation::intgen, valuegen},
    frontend::{
        types::ast::Ast,
        typesystem::{traits::LLVMTypeExtensions, types::Type},
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
            if expr_type.llvm_is_ptr_type() {
                let depth: IntValue =
                    valuegen::compile(context, index, Some(&Type::U64)).into_int_value();

                vec![depth]
            } else {
                let base: IntValue = intgen::int(llvm_context, &Type::U32, 0, false);

                let depth: IntValue =
                    valuegen::compile(context, index, Some(&Type::U32)).into_int_value();

                vec![base, depth]
            }
        })
        .collect()
}
