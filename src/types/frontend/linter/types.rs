use ahash::AHashMap as HashMap;

use crate::frontend::lexer::span::Span;

pub type LinterConstantInfo = (Span, bool);
pub type LinterConstants<'linter> = HashMap<&'linter str, LinterConstantInfo>;

pub type LinterLLIInfo<'symbol> = (Span, bool);
pub type LinterLLIs<'symbol> = Vec<HashMap<&'symbol str, LinterLLIInfo<'symbol>>>;

pub type LinterFunctionInfo<'linter> = (Span, bool);
pub type LinterFunctions<'linter> = HashMap<&'linter str, LinterFunctionInfo<'linter>>;

pub type LinterLocalInfo = (Span, bool, bool);
pub type LinterLocals<'linter> = Vec<HashMap<&'linter str, LinterLocalInfo>>;

pub type LinterFunctionParameterInfo = (Span, bool, bool);
pub type LinterFunctionParameters<'linter> = HashMap<&'linter str, LinterFunctionParameterInfo>;
