use crate::{
    backend::llvm::compiler::anchors::PointerAnchor,
    frontend::types::{
        lexer::Type,
        parser::{repr::Local, stmts::types::ThrushAttributes},
    },
};

use super::{Ast, context::LLVMCodeGenContext, memory::SymbolAllocated, valuegen};

use inkwell::values::BasicValueEnum;

pub fn new<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, local: Local<'ctx>) {
    let local_name: &str = local.0;
    let ascii_name: &str = local.1;

    let local_type: &Type = local.2;
    let local_value: &Ast = local.3;

    let attributes: &ThrushAttributes = local.4;

    context.new_local(local_name, ascii_name, local_type, attributes);

    let symbol: SymbolAllocated = context.get_symbol(local_name);

    context.set_pointer_anchor(PointerAnchor::new(
        symbol.get_value().into_pointer_value(),
        false,
    ));

    let value: BasicValueEnum = valuegen::compile(context, local_value, Some(local_type));

    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            symbol.store(context, value);
        }
    } else {
        symbol.store(context, value);
    }

    context.clear_pointer_anchor();
}
