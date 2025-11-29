use crate::{core::diagnostic::span::Span, front_end::typesystem::types::Type};

use ahash::AHashMap as HashMap;

pub type AnalyzerLocal<'symbol> = &'symbol Type;
pub type AnalyzerLocals<'symbol> = Vec<HashMap<&'symbol str, AnalyzerLocal<'symbol>>>;

pub type AnalyzerLLI<'symbol> = (&'symbol Type, Span);
pub type AnalyzerLLIs<'symbol> = Vec<HashMap<&'symbol str, AnalyzerLLI<'symbol>>>;

pub type AnalyzerAssemblerFunction<'symbol> = (&'symbol [Type], bool);
pub type AnalyzerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, AnalyzerAssemblerFunction<'symbol>>;

pub type AnalyzerFunction<'symbol> = (&'symbol [Type], bool);
pub type AnalyzerFunctions<'symbol> = HashMap<&'symbol str, AnalyzerFunction<'symbol>>;
