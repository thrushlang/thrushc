use crate::back_end::llvm::compiler::attributes::LLVMAttribute;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::types::Type;

pub type StructFields<'ctx> = (
    &'ctx str,
    Vec<(&'ctx str, Type, u32, Span)>,
    StructureTypeModificator,
);
pub type StructField<'ctx> = (usize, &'ctx (&'ctx str, Type, u32, Span));

pub type EnumFields<'ctx> = Vec<(&'ctx str, Type, Ast<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Type, Ast<'ctx>);

pub type Constructor<'ctx> = Vec<(&'ctx str, Ast<'ctx>, Type, u32)>;

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
