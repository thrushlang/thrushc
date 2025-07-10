use crate::{
    backend::llvm::compiler::{
        anchors::PointerAnchor, codegen, context::LLVMCodeGenContext, memory::SymbolAllocated,
    },
    frontend::{
        types::{
            ast::Ast,
            parser::{repr::Local, stmts::types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, local: Local<'ctx>) {
    let name: &str = local.0;
    let ascii_name: &str = local.1;

    let local_type: &Type = local.2;
    let expr: &Ast = local.3;

    let attributes: &ThrushAttributes = local.4;

    context.new_local(name, ascii_name, local_type, attributes);

    let symbol: SymbolAllocated = context.get_table().get_symbol(name);

    context.set_pointer_anchor(PointerAnchor::new(symbol.get_ptr(), false));

    let value: BasicValueEnum = codegen::compile_expr(context, expr, Some(local_type), false);

    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            symbol.store(context, value);
        }
    } else {
        symbol.store(context, value);
    }

    context.clear_pointer_anchor();
}
