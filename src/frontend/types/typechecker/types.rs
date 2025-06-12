use ahash::AHashMap as HashMap;

use crate::frontend::{lexer::span::Span, types::lexer::ThrushType};

pub type TypeCheckerLocal<'symbol> = &'symbol ThrushType;
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerLLI<'symbol> = (&'symbol ThrushType, Span);
pub type TypeCheckerLLIs<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLLI<'symbol>>>;

pub type TypeCheckerAssemblerFunction<'symbol> = (&'symbol [ThrushType], bool);
pub type TypeCheckerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, TypeCheckerAssemblerFunction<'symbol>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol [ThrushType], bool);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;
