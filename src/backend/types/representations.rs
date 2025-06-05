use inkwell::values::FunctionValue;

use crate::frontend::types::lexer::ThrushType;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [ThrushType], u32);
