use crate::{
    backends::classical::llvm::compiler::{constants, constgen, context::LLVMCodeGenContext},
    frontends::classical::{
        types::{
            ast::{Ast, metadata::staticvar::StaticMetadata},
            parser::repr::LocalStatic,
        },
        typesystem::types::Type,
    },
};

use inkwell::values::BasicValueEnum;

pub fn compile_local<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    staticvar: LocalStatic<'ctx>,
) {
    let name: &str = staticvar.0;
    let ascii_name: &str = staticvar.1;

    let kind: &Type = staticvar.2;
    let expr: &Ast = staticvar.3;

    let metadata: StaticMetadata = staticvar.4;

    let expr_type: &Type = expr.get_type_unwrapped();

    let llvm_value: BasicValueEnum = constgen::compile(context, expr, kind);
    let value: BasicValueEnum = constants::casts::try_one(context, llvm_value, expr_type, kind);

    context.new_local_static(name, ascii_name, kind, value, metadata);
}
