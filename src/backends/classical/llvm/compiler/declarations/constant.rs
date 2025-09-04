use crate::backends::classical::llvm::compiler::constants;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::ast::metadata::constant::ConstantMetadata;
use crate::frontends::classical::types::parser::repr::GlobalConstant;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

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
    let metadata: ConstantMetadata = constant.5;

    let llvm_value: BasicValueEnum = constgen::compile(context, value, kind);
    let value_type: &Type = value.get_type_unwrapped();

    let value: BasicValueEnum = constants::casts::try_one(context, llvm_value, value_type, kind);

    context.new_global_constant(name, ascii_name, kind, value, attributes, metadata);
}
