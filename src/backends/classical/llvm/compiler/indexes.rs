use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::generation::int;
use crate::backends::classical::llvm::compiler::value;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::traits::LLVMTypeExtensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::context::Context;
use inkwell::values::IntValue;

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [Ast],
    expr_type: &'ctx Type,
) -> Vec<IntValue<'ctx>> {
    let llvm_context: &Context = context.get_llvm_context();

    let index_type: &Type = if expr_type.llvm_is_ptr_type() {
        &Type::U64
    } else {
        &Type::U32
    };

    indexes
        .iter()
        .flat_map(|index| {
            if expr_type.llvm_is_ptr_type() {
                let depth: IntValue =
                    value::compile(context, index, Some(index_type)).into_int_value();

                vec![depth]
            } else {
                let base: IntValue = int::generate(llvm_context, index_type, 0, false);
                let depth: IntValue =
                    value::compile(context, index, Some(index_type)).into_int_value();

                vec![base, depth]
            }
        })
        .collect()
}
