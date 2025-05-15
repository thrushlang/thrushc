use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::Span, middle::types::frontend::parser::stmts::instruction::Instruction,
};

pub type WarnerConstantInfo = (Span, bool);
pub type WarnersConstants<'warner> = HashMap<&'warner str, WarnerConstantInfo>;

pub type WarnerFunctionInfo<'warner> = (&'warner Instruction<'warner>, Span, bool);
pub type WarnersFunctions<'warner> = HashMap<&'warner str, WarnerFunctionInfo<'warner>>;

pub type WarnerLocalInfo = (Span, bool, bool);
pub type WarnerLocals<'warner> = Vec<HashMap<&'warner str, WarnerLocalInfo>>;

pub type WarnerFunctionParameterInfo = (Span, bool, bool);
pub type WarnersFunctionParameters<'warner> = HashMap<&'warner str, WarnerFunctionParameterInfo>;
