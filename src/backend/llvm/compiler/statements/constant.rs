use crate::{
    backend::llvm::compiler::{constants, constgen, context::LLVMCodeGenContext},
    frontend::{
        types::{
            ast::{Ast, metadata::constant::ConstantMetadata},
            parser::repr::LocalConstant,
        },
        typesystem::types::Type,
    },
};

use inkwell::values::BasicValueEnum;

pub fn compile_local<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    constant: LocalConstant<'ctx>,
) {
    let name: &str = constant.0;
    let ascii_name: &str = constant.1;

    let kind: &Type = constant.2;
    let expr: &Ast = constant.3;

    let metadata: ConstantMetadata = constant.4;

    let expr_type: &Type = expr.get_type_unwrapped();

    let compiled_value: BasicValueEnum = constgen::compile(context, expr, kind);
    let value: BasicValueEnum = constants::casts::try_one(context, compiled_value, expr_type, kind);

    context.new_local_constant(name, ascii_name, kind, value, metadata);
}
