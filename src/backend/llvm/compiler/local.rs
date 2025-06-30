use crate::{
    backend::llvm::compiler::anchors::PointerAnchor,
    frontend::types::{
        lexer::ThrushType,
        parser::{repr::Local, stmts::types::ThrushAttributes},
    },
};

use super::{Ast, context::LLVMCodeGenContext, memory::SymbolAllocated, valuegen};

use inkwell::values::BasicValueEnum;

pub fn new<'ctx>(local: Local<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let ascii_name: &str = local.1;

    let local_type: &ThrushType = local.2;
    let local_value: &Ast = local.3;

    let attributes: &ThrushAttributes = local.4;

    context.alloc_local(local_name, ascii_name, local_type, attributes);

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    context.set_pointer_anchor(PointerAnchor::new(
        symbol.get_value().into_pointer_value(),
        false,
    ));

    let value: BasicValueEnum = valuegen::compile(context, local_value, Some(local_type));

    context.clear_pointer_anchor();

    if !value.is_pointer_value() {
        symbol.store(context, value);
    }
}
