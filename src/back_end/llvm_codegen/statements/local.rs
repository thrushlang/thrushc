use crate::back_end::llvm_codegen::anchors::PointerAnchor;
use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::memory::SymbolAllocated;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::repr::Local;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, local: Local<'ctx>) {
    let name: &str = local.0;

    let kind: &Type = local.2;
    let expr: Option<&Ast> = local.3;

    context.new_local(local);

    if let Some(expr) = expr {
        let symbol: SymbolAllocated = context.get_table().get_symbol(name);

        context.set_pointer_anchor(PointerAnchor::new(symbol.get_ptr(), false));

        let value: BasicValueEnum = codegen::compile(context, expr, Some(kind));

        if let Some(anchor) = context.get_pointer_anchor() {
            if !anchor.is_triggered() {
                symbol.store(context, value);
            }
        } else {
            symbol.store(context, value);
        }

        context.clear_pointer_anchor();
    }
}
