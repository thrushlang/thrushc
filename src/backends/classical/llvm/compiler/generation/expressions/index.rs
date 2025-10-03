use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::indexes;
use crate::backends::classical::llvm::compiler::memory;
use crate::backends::classical::llvm::compiler::ptr;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.get_type_unwrapped();

    let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, ptr_type);

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes).into()
}
