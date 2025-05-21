use ahash::AHashMap as HashMap;

use crate::types::frontend::lexer::types::ThrushType;

pub type TypeCheckerLocal<'symbol> = &'symbol ThrushType;
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol [ThrushType], bool);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;

pub type TypeCheckerBind<'symbol> = &'symbol [ThrushType];
pub type TypeCheckerBinds<'symbol> = Vec<(&'symbol str, TypeCheckerBind<'symbol>)>;
pub type TypeCheckerBindings<'symbol> = HashMap<&'symbol str, TypeCheckerBinds<'symbol>>;
