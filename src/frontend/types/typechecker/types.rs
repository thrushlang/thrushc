use ahash::AHashMap as HashMap;

use crate::frontend::{lexer::span::Span, types::lexer::ThrushType};

pub type TypeCheckerLocal<'symbol> = &'symbol ThrushType;
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerLLI<'symbol> = (&'symbol ThrushType, Span);
pub type TypeCheckerLLIs<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLLI<'symbol>>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol [ThrushType], bool);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;

pub type TypeCheckerMethod<'symbol> = &'symbol [ThrushType];
pub type TypeCheckerAllMethods<'symbol> = Vec<(&'symbol str, TypeCheckerMethod<'symbol>)>;
pub type TypeCheckerMethods<'symbol> = HashMap<&'symbol str, TypeCheckerAllMethods<'symbol>>;
