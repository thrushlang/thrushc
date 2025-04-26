pub mod impls;
pub mod traits;

use std::rc::Rc;

use crate::backend::llvm::compiler::{attributes::LLVMAttribute, memory::MemoryFlag};

use inkwell::values::FunctionValue;

use super::{
    instruction::Instruction,
    types::{TokenKind, Type},
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Type, &'ctx Instruction<'ctx>);

pub type Local<'ctx> = (&'ctx str, &'ctx Type, &'ctx Instruction<'ctx>, MemoryFlags);

pub type FunctionCall<'ctx> = (&'ctx str, &'ctx Type, &'ctx [Instruction<'ctx>]);

pub type FunctionPrototype<'ctx> = (
    &'ctx str,
    &'ctx Type,
    &'ctx [Instruction<'ctx>],
    Option<&'ctx Rc<Instruction<'ctx>>>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type Function<'ctx> = (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>], u32);
pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx Type, u32, MemoryFlags);

pub type StructFields<'ctx> = Vec<(&'ctx str, Type, u32)>;

pub type Enum<'ctx> = (EnumFields<'ctx>, ThrushAttributes<'ctx>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, Instruction<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Instruction<'ctx>);

pub type CustomType<'ctx> = (CustomTypeFields<'ctx>, ThrushAttributes<'ctx>);
pub type CustomTypeField<'ctx> = Type;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'instr> = Vec<(&'instr str, Instruction<'instr>, Type, u32)>;

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;

pub type MemoryFlags = [MemoryFlag; 1];
