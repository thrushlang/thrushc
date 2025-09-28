use inkwell::values::PointerValue;

#[derive(Debug, Clone, Copy)]
pub struct PointerAnchor<'ctx> {
    pub pointer: PointerValue<'ctx>,
    pub triggered: bool,
}

impl<'ctx> PointerAnchor<'ctx> {
    #[inline]
    pub fn new(pointer: PointerValue<'ctx>, triggered: bool) -> PointerAnchor<'ctx> {
        Self { pointer, triggered }
    }

    #[inline]
    pub fn get_pointer(&self) -> PointerValue<'ctx> {
        self.pointer
    }

    #[inline]
    pub fn is_triggered(&self) -> bool {
        self.triggered
    }
}
