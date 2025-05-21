use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::span::Span, middle::types::frontend::parser::stmts::stmt::ThrushStatement,
};

pub type LinterConstantInfo = (Span, bool);
pub type LinterConstants<'warner> = HashMap<&'warner str, LinterConstantInfo>;

pub type LinterFunctionInfo<'warner> = (&'warner ThrushStatement<'warner>, Span, bool);
pub type LinterFunctions<'warner> = HashMap<&'warner str, LinterFunctionInfo<'warner>>;

pub type LinterLocalInfo = (Span, bool, bool);
pub type LinterLocals<'warner> = Vec<HashMap<&'warner str, LinterLocalInfo>>;

pub type LinterFunctionParameterInfo = (Span, bool, bool);
pub type LinterFunctionParameters<'warner> = HashMap<&'warner str, LinterFunctionParameterInfo>;
