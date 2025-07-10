use crate::{
    backend::llvm::compiler::{constgen, context::LLVMCodeGenContext},
    frontend::{
        types::{
            ast::{Ast, metadata::staticvar::StaticMetadata},
            parser::{repr::GlobalStatic, stmts::types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

use inkwell::values::BasicValueEnum;

pub fn compile_global<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    staticvar: GlobalStatic<'ctx>,
) {
    let name: &str = staticvar.0;
    let ascii_name: &str = staticvar.1;

    let kind: &Type = staticvar.2;
    let value: &Ast = staticvar.3;

    let metadata: StaticMetadata = staticvar.4;
    let attributes: &ThrushAttributes = staticvar.5;

    let value_type: &Type = value.get_type_unwrapped();

    let llvm_value: BasicValueEnum = constgen::compile(context, value, kind);
    let value: BasicValueEnum = constgen::cast(context, llvm_value, value_type, kind);

    context.new_global_static(name, ascii_name, kind, value, metadata, attributes);
}
