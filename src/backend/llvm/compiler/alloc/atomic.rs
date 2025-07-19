use inkwell::{AtomicOrdering, values::InstructionValue};

#[derive(Debug, Clone, Copy)]
pub struct LLVMAtomicModificators {
    pub atomic_volatile: bool,
    pub atomic_ord: Option<AtomicOrdering>,
}

pub fn try_set_atomic_modificators(
    instr: InstructionValue<'_>,
    modificators: LLVMAtomicModificators,
) {
    if modificators.atomic_volatile {
        let _ = instr.set_volatile(true);
    }

    if let Some(ord) = modificators.atomic_ord {
        let _ = instr.set_atomic_ordering(ord);
    }
}
