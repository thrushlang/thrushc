use crate::backend::llvm::compiler::codegen;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::generation::integer;

use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::traits::LLVMTypeExtensions;
use crate::frontend::typesystem::types::Type;

use inkwell::values::IntValue;

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [Ast],
    expr_type: &'ctx Type,
) -> Vec<IntValue<'ctx>> {
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
