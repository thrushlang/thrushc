use crate::frontends::classical::{
    types::parser::stmts::types::ThrushAttributes,
    typesystem::modificators::{
        GCCStructureTypeModificator, LLVMStructureTypeModificator, StructureTypeModificator,
    },
};

#[inline]
pub fn generate_structure_modificator(attributes: &ThrushAttributes) -> StructureTypeModificator {
    let llvm_packed_modificator: bool = attributes.iter().any(|attr| attr.is_packed());

    StructureTypeModificator::new(
        LLVMStructureTypeModificator::new(llvm_packed_modificator),
        GCCStructureTypeModificator::new(),
    )
}
