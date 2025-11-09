use crate::back_end::llvm::compiler::codegen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::generation::integer;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::IntValue;

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [Ast],
    expr_type: &'ctx Type,
) -> Vec<IntValue<'ctx>> {
    let is_ptr_type: bool = expr_type.is_ptr_like_type();

    let index_type: &Type = if is_ptr_type { &Type::U64 } else { &Type::U32 };

    indexes
        .iter()
        .flat_map(|index| {
            if is_ptr_type {
                let depth: IntValue =
                    codegen::compile(context, index, Some(index_type)).into_int_value();

                vec![depth]
            } else {
                let base: IntValue =
                    integer::generate(context, index_type, 0, false, index.get_span());
                let depth: IntValue =
                    codegen::compile(context, index, Some(index_type)).into_int_value();

                vec![base, depth]
            }
        })
        .collect()
}
