use {
    super::{
        super::super::{
            backend::compiler::attributes::CompilerAttribute,
            frontend::lexer::{TokenKind, Type},
        },
        Instruction,
        memory::{AllocatedObject, MemoryFlag},
    },
    ahash::AHashMap as HashMap,
    inkwell::values::FunctionValue,
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Instruction<'ctx>, &'ctx Type);

pub type Local<'ctx> = (&'ctx str, &'ctx Type, &'ctx Instruction<'ctx>, MemoryFlags);

pub type Call<'ctx> = (&'ctx str, &'ctx Type, &'ctx [Instruction<'ctx>]);

pub type Function<'ctx> = (
    &'ctx str,
    &'ctx Type,
    &'ctx [Instruction<'ctx>],
    Option<&'ctx Box<Instruction<'ctx>>>,
    &'ctx [CompilerAttribute<'ctx>],
);

pub type CompilerFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>], u32);
pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx str, &'ctx Type, u32, MemoryFlags);

pub type CompilerStructure<'ctx> = (&'ctx str, Vec<(&'ctx str, &'ctx str, Type, u32)>);
pub type CompilerStructureFields<'ctx> = Vec<(&'ctx str, &'ctx str, Type, u32)>;

pub type CompilerAttributes<'ctx> = Vec<CompilerAttribute<'ctx>>;

pub type AllocatedObjects<'ctx> = &'ctx HashMap<&'ctx str, AllocatedObject<'ctx>>;

pub type MemoryFlags = Vec<MemoryFlag>;
