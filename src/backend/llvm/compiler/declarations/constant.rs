use crate::{
    backend::llvm::compiler::{constgen, context::LLVMCodeGenContext},
    frontend::{
        types::{
            ast::Ast,
            parser::{repr::GlobalConstant, stmts::types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

use inkwell::values::BasicValueEnum;

pub fn compile_global<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    constant: GlobalConstant<'ctx>,
) {
    let name: &str = constant.0;
    let ascii_name: &str = constant.1;

    let kind: &Type = constant.2;
    let value: &Ast = constant.3;
    let attributes: &ThrushAttributes = constant.4;

    let llvm_value: BasicValueEnum = constgen::compile(context, value, kind);
    let value_type: &Type = value.get_type_unwrapped();

    let value: BasicValueEnum = constgen::cast(context, llvm_value, value_type, kind);

    context.new_global_constant(name, ascii_name, kind, value, attributes);
}
