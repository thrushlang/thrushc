#[derive(Debug, Clone, Copy)]
pub enum ReferenceIndentificator {
    FunctionParameter,
    Constant,
    Local,
    LowLevelInstruction,
}

impl ReferenceIndentificator {
    #[inline]
    pub fn is_function_parameter(&self) -> bool {
        matches!(self, ReferenceIndentificator::FunctionParameter)
    }

    #[inline]
    pub fn is_constant(&self) -> bool {
        matches!(self, ReferenceIndentificator::Constant)
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        matches!(self, ReferenceIndentificator::Local)
    }

    #[inline]
    pub fn is_lli(&self) -> bool {
        matches!(self, ReferenceIndentificator::LowLevelInstruction)
    }
}
