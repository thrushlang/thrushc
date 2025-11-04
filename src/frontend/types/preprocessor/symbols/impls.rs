use crate::frontend::{
    preprocessor::types::FoundModuleSymbolId,
    types::preprocessor::symbols::traits::{FoundModuleSymbolEither, FoundModuleSymbolEntension},
};

impl FoundModuleSymbolEntension for FoundModuleSymbolId {
    fn is_structure(&self) -> bool {
        self.0.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.1.is_some()
    }
}

impl<'symbols> FoundModuleSymbolEither<'symbols> for FoundModuleSymbolId {
    fn expected_structure(&self) -> Result<String, ()> {
        if let Some(structure) = self.0.clone() {
            return Ok(structure);
        }

        Err(())
    }

    fn expected_custom_type(&self) -> Result<String, ()> {
        if let Some(cstype) = self.1.clone() {
            return Ok(cstype);
        }

        Err(())
    }
}
