use crate::{core::diagnostic::span::Span, front_end::typesystem::types::Type};

use ahash::AHashMap as HashMap;

pub type TypeCheckerLocal<'symbol> = &'symbol Type;
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerLLI<'symbol> = (&'symbol Type, Span);
pub type TypeCheckerLLIs<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLLI<'symbol>>>;

pub type TypeCheckerIntrinsic<'symbol> = (&'symbol [Type], bool);
pub type TypeCheckerIntrinsics<'symbol> = HashMap<&'symbol str, TypeCheckerIntrinsic<'symbol>>;

pub type TypeCheckerAssemblerFunction<'symbol> = (&'symbol [Type], bool);
pub type TypeCheckerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, TypeCheckerAssemblerFunction<'symbol>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol [Type], bool);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;
