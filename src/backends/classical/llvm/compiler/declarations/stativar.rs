use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontends::classical::types::parser::repr::GlobalStatic;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub fn compile_global<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    staticvar: GlobalStatic<'ctx>,
) {
    let name: &str = staticvar.0;
    let ascii_name: &str = staticvar.1;

    let kind: &Type = staticvar.2;
    let value: &Ast = staticvar.3;

    let attributes: &ThrushAttributes = staticvar.4;
    let metadata: StaticMetadata = staticvar.5;

    let value_type: &Type = value.get_type_unwrapped();

    let llvm_value: BasicValueEnum = compiler::constgen::compile(context, value, kind);
    let value: BasicValueEnum =
        compiler::generation::cast::try_cast_const(context, llvm_value, value_type, kind);

    context.new_global_static(name, ascii_name, kind, value, attributes, metadata);
}
