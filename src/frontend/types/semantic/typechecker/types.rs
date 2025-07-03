use ahash::AHashMap as HashMap;

use crate::frontend::{lexer::span::Span, types::lexer::Type};

pub type TypeCheckerLocal<'symbol> = &'symbol Type;
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerLLI<'symbol> = (&'symbol Type, Span);
pub type TypeCheckerLLIs<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLLI<'symbol>>>;

pub type TypeCheckerAssemblerFunction<'symbol> = (&'symbol [Type], bool);
pub type TypeCheckerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, TypeCheckerAssemblerFunction<'symbol>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol [Type], bool);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;
