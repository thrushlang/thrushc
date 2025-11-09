use crate::front_end::{
    preprocessor::types::{
        CustomTypeSymbol, FoundModuleSymbolId, GlobalCustomTypes, GlobalStructs, StructSymbol,
    },
    typesystem::types::Type,
};

#[derive(Debug)]
pub struct ModuleSymbolTable {
    global_custom_types: GlobalCustomTypes,
    global_structs: GlobalStructs,
}

impl ModuleSymbolTable {
    #[inline]
    pub fn new() -> Self {
        Self {
            global_custom_types: GlobalCustomTypes::with_capacity(100),
            global_structs: GlobalStructs::with_capacity(100),
        }
    }
}

impl ModuleSymbolTable {
    #[inline]
    pub fn new_struct(&mut self, name: String, kind: Type) {
        if !self.global_structs.contains_key(&name) {
            self.global_structs.insert(name, kind);
        }
    }

    #[inline]
    pub fn new_custom_type(&mut self, name: String, kind: Type) {
        if !self.global_custom_types.contains_key(&name) {
            self.global_custom_types.insert(name, kind);
        }
    }
}

impl ModuleSymbolTable {
    #[inline]
    pub fn get_struct_by_id(&self, id: String) -> Result<StructSymbol, ()> {
        if let Some(kind) = self.global_structs.get(&id) {
            return Ok(kind.clone());
        }

        Err(())
    }

    #[inline]
    pub fn get_custom_type_by_id(&self, id: String) -> Result<CustomTypeSymbol, ()> {
        if let Some(kind) = self.global_custom_types.get(&id) {
            return Ok(kind.clone());
        }

        Err(())
    }
}

impl ModuleSymbolTable {
    #[inline]
    pub fn get_symbols_id(&self, id: String) -> Result<FoundModuleSymbolId, ()> {
        if self.global_structs.contains_key(&id) {
            return Ok((Some(id), None));
        }

        if self.global_custom_types.contains_key(&id) {
            return Ok((None, Some(id)));
        }

        Err(())
    }
}
