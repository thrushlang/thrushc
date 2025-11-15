#[derive(Debug, Clone, Copy)]
pub enum AttributeCheckerAttributeApplicant {
    AssemblerFunction,
    Intrinsic,
    Function,
    Constant,
    Static,
    Struct,
    Enum,
    Local,
}
