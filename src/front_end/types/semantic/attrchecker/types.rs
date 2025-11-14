#[derive(Debug, Clone, Copy)]
pub enum AttributeCheckerAttributeApplicant {
    AssemblerFunction,
    Function,
    Constant,
    Static,
    Struct,
    Enum,
}
