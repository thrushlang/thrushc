#[derive(Debug, Clone, Copy)]
pub enum ReferenceIdentificator {
    FunctionParameter,
    Constant,
    Local,
    LowLevelInstruction,
}
