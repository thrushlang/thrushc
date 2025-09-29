use ahash::AHashMap as HashMap;

use crate::frontends::classical::lexer::span::Span;

pub type LinterStaticInfo = (Span, bool, bool);
pub type LinterGlobalStatics<'linter> = HashMap<&'linter str, LinterStaticInfo>;
pub type LinterLocalStatics<'linter> = Vec<HashMap<&'linter str, LinterStaticInfo>>;

pub type LinterConstantInfo = (Span, bool);
pub type LinterGlobalConstants<'linter> = HashMap<&'linter str, LinterConstantInfo>;
pub type LinterLocalConstants<'linter> = Vec<HashMap<&'linter str, LinterConstantInfo>>;

pub type LinterLLIInfo<'symbol> = (Span, bool);
pub type LinterLLIs<'symbol> = Vec<HashMap<&'symbol str, LinterLLIInfo<'symbol>>>;

pub type LinterAssemblerFunctionInfo<'linter> = (Span, bool);
pub type LinterAssemblerFunctions<'linter> =
    HashMap<&'linter str, LinterAssemblerFunctionInfo<'linter>>;

pub type LinterFunctionInfo<'linter> = (Span, bool);
pub type LinterFunctions<'linter> = HashMap<&'linter str, LinterFunctionInfo<'linter>>;

pub type LinterLocalInfo = (Span, bool, bool);
pub type LinterLocals<'linter> = Vec<HashMap<&'linter str, LinterLocalInfo>>;

pub type LinterEnumFieldInfo = (Span, bool);

pub type LinterEnumsFieldsInfo<'linter> = (HashMap<&'linter str, LinterEnumFieldInfo>, Span, bool);
pub type LinterEnums<'linter> = HashMap<&'linter str, LinterEnumsFieldsInfo<'linter>>;

pub type LinterStructFieldInfo = (Span, bool);
pub type LinterStructFieldsInfo<'linter> =
    (HashMap<&'linter str, LinterStructFieldInfo>, Span, bool);
pub type LinterStructs<'linter> = HashMap<&'linter str, LinterStructFieldsInfo<'linter>>;

pub type LinterFunctionParameterInfo = (Span, bool, bool);
pub type LinterFunctionParameters<'linter> =
    Vec<HashMap<&'linter str, LinterFunctionParameterInfo>>;

#[derive(Debug, Clone, Copy)]
pub enum LinterAttributeApplicant {
    Function,
    Struct,
    Constant,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LLVMAttributeComparator {
    Extern,
    Convention,
    Public,
    Ignore,
    Hot,
    NoInline,
    InlineHint,
    MinSize,
    AlwaysInline,
    SafeStack,
    StrongStack,
    WeakStack,
    PreciseFloats,
    NoUnwind,
    OptFuzzing,

    Stack,
    Heap,

    AsmThrow,
    AsmSyntax,
    AsmAlignStack,
    AsmSideEffects,

    Packed,
}
