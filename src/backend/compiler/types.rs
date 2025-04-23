use super::super::super::{
    backend::compiler::attributes::LLVMAttribute,
    frontend::lexer::{TokenKind, Type},
};

use super::{
    Instruction,
    memory::{AllocatedObject, MemoryFlag},
};

use {
    ahash::{AHashMap as HashMap, HashSet},
    inkwell::values::FunctionValue,
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
    &'ctx Instruction<'ctx>,
);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx Instruction<'ctx>,
    &'ctx Instruction<'ctx>,
    MemoryFlags,
);

pub type FunctionCall<'ctx> = (
    &'ctx str,
    &'ctx Instruction<'ctx>,
    &'ctx [Instruction<'ctx>],
);

pub type FunctionPrototype<'ctx> = (
    &'ctx str,
    &'ctx Instruction<'ctx>,
    &'ctx [Instruction<'ctx>],
    Option<&'ctx Box<Instruction<'ctx>>>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type Function<'ctx> = (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>], u32);
pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx Instruction<'ctx>, u32, MemoryFlags);

pub type Structure<'ctx> = (&'ctx str, Vec<(&'ctx str, Instruction<'ctx>, u32)>);
pub type StructureFields<'ctx> = Vec<(&'ctx str, Instruction<'ctx>, u32)>;

pub type EnumFields<'ctx> = Vec<(&'ctx str, Type, Instruction<'ctx>)>;

pub type FixedArray = (Type, u32);

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;

pub type AllocatedObjects<'ctx> = &'ctx HashMap<&'ctx str, AllocatedObject<'ctx>>;
pub type MappedHeapPointers<'ctx> = HashSet<(&'ctx str, u32, bool)>;
pub type MappedHeapPointer<'ctx> = (&'ctx str, u32, bool);

pub type MemoryFlags = [MemoryFlag; 1];
