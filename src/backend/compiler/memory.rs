use inkwell::{
    builder::Builder,
    types::BasicType,
    values::{BasicValue, BasicValueEnum, InstructionValue, PointerValue},
};

#[derive(Debug, Clone, Copy)]
pub struct AllocatedObject<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub memory_flags: u8,
}

impl<'ctx> AllocatedObject<'ctx> {
    pub fn alloc(ptr: PointerValue<'ctx>, initial_flags: &[MemoryFlag]) -> Self {
        let mut memory_flags: u8 = 0;

        initial_flags.iter().for_each(|flag| {
            memory_flags |= flag.to_bit();
        });

        Self { ptr, memory_flags }
    }

    pub fn load_from_memory<Type: BasicType<'ctx>>(
        &self,
        builder: &Builder<'ctx>,
        llvm_type: Type,
    ) -> BasicValueEnum<'ctx> {
        if self.has_flag(MemoryFlag::StackAllocated) {
            let load: BasicValueEnum = builder.build_load(llvm_type, self.ptr, "").unwrap();

            if let Some(load_instruction) = load.as_instruction_value() {
                let _ = load_instruction.set_alignment(8);
            }

            return load;
        }

        self.ptr.into()
    }

    pub fn build_store<Value: BasicValue<'ctx>>(&self, builder: &Builder<'ctx>, value: Value) {
        let store: InstructionValue = builder.build_store(self.ptr, value).unwrap();

        let _ = store.set_alignment(8);
    }

    fn has_flag(&self, flag: MemoryFlag) -> bool {
        (self.memory_flags & flag.to_bit()) == flag.to_bit()
    }
}

#[derive(Debug)]
pub enum MemoryFlag {
    StackAllocated,
    HeapAllocated,
    StaticAllocated,
}

impl MemoryFlag {
    #[inline(always)]
    pub fn to_bit(&self) -> u8 {
        match self {
            MemoryFlag::StackAllocated => 1 << 0,
            MemoryFlag::HeapAllocated => 1 << 1,
            MemoryFlag::StaticAllocated => 1 << 2,
        }
    }
}
