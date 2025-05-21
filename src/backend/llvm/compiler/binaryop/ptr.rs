use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, predicates, valuegen},
    middle::types::{
        backend::llvm::types::LLVMBinaryOp,
        frontend::{
            lexer::{tokenkind::TokenKind, types::ThrushType},
            parser::stmts::stmt::ThrushStatement,
        },
    },
};

pub fn ptr_operation<'ctx>(
    builder: &Builder<'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenKind,
) -> BasicValueEnum<'ctx> {
    if left.is_pointer_value() && right.is_pointer_value() {
        let left: PointerValue = left.into_pointer_value();
        let right: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_type() => builder
                .build_int_compare(predicates::pointer(operator), left, right, "")
                .unwrap()
                .into(),
            _ => unreachable!(),
        };
    }

    unreachable!()
}

pub fn ptr_binaryop<'ctx>(
    binary: LLVMBinaryOp<'ctx>,
    target_type: &ThrushType,
    symbols: &mut LLVMCodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let context: &Context = symbols.get_llvm_context();
    let builder: &Builder = symbols.get_llvm_builder();

    /* ######################################################################


        CALL BINARY - BINARY EXPRESSIONS


    ########################################################################*/

    if let (
        ThrushStatement::Call { .. },
        TokenKind::EqEq | TokenKind::BangEq,
        ThrushStatement::NullPtr { .. },
    ) = binary
    {
        let left_compiled: BasicValueEnum = valuegen::build(binary.0, target_type, symbols);
        let right_compiled: PointerValue = context.ptr_type(AddressSpace::default()).const_null();

        return ptr_operation(builder, left_compiled, right_compiled.into(), binary.1);
    }

    if let (
        ThrushStatement::NullPtr { .. },
        TokenKind::EqEq | TokenKind::BangEq,
        ThrushStatement::Call { .. },
    ) = binary
    {
        let left_compiled: PointerValue = context.ptr_type(AddressSpace::default()).const_null();
        let right_compiled: BasicValueEnum = valuegen::build(binary.2, target_type, symbols);

        return ptr_operation(builder, left_compiled.into(), right_compiled, binary.1);
    }

    todo!()
}
