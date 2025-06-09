#[derive(Debug, Clone, Copy)]
pub enum ReferenceIndentificator {
    FunctionParameter,
    Constant,
    Local,
    LowLevelInstruction,
}
