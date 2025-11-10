use std::{fs, path::Path};

use object::{Object, ObjectSymbol, SymbolKind};

use crate::linkage::checker::signatures::LinkageCheckerSignature;

pub fn is_dyn_library(library_path: &Path) -> bool {
    library_path
        .extension()
        .is_some_and(|ext| ext == "so" || ext == "dll")
}

pub fn find_symbol<'symbol>(
    library_path: &Path,
    signature: LinkageCheckerSignature<'symbol>,
) -> Result<bool, ()> {
    let library_data: Vec<u8> = fs::read(library_path).map_err(|_| ())?;
    let object: object::File<'_> = object::File::parse(library_data.as_slice()).map_err(|_| ())?;

    for symbol in object.symbols() {
        if let Ok(symbol_name) = symbol.name_bytes() {
            let symbol_type: SymbolKind = symbol.kind();

            if symbol_name == signature.name
                && symbol.is_definition()
                && ((signature.variant.is_function() && symbol_type == SymbolKind::Text)
                    || (signature.variant.is_global() && symbol_type == SymbolKind::Data))
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
