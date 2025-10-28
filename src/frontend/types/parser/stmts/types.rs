use crate::backend::llvm::compiler::attributes::LLVMAttribute;

use crate::frontend::lexer::span::Span;
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::modificators::StructureTypeModificator;
use crate::frontend::typesystem::types::Type;

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
