use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::memory;
use crate::back_end::llvm_codegen::refptr;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstLLVMGetType;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let indexes: Vec<IntValue> = indexes
        .iter()
        .map(|index| {
            codegen::compile(context, index, Some(&Type::U32(source.get_span()))).into_int_value()
        })
        .collect();

    let span: Span = source.get_span();
    let kind: &Type = source.llvm_get_type(context);
    let ptr: PointerValue = refptr::compile(context, source, None).into_pointer_value();

    memory::gep_anon(context, ptr, kind, &indexes, span).into()
}
