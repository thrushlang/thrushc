use std::{fs, path::Path};

use object::{Object, ObjectSymbol, SymbolKind, read::archive::ArchiveFile};

use crate::linkage::checker::signatures::LinkageCheckerSignature;

pub fn is_static_library(library_path: &Path) -> bool {
    library_path
        .extension()
        .is_some_and(|ext| ext == "a" || ext == "lib")
}

pub fn find_symbol<'symbol>(
    library_path: &Path,
    signature: LinkageCheckerSignature<'symbol>,
) -> Result<bool, ()> {
    let library_data: Vec<u8> = fs::read(library_path).map_err(|_| ())?;
    let library: ArchiveFile = ArchiveFile::parse(library_data.as_slice()).map_err(|_| ())?;

    for member in library.members().flatten() {
        if !member.is_thin() {
            if let Ok(member_data) = member.data(library_data.as_slice()) {
                if let Ok(object) = object::File::parse(member_data) {
                    for symbol in object.symbols() {
                        let symbol_type: SymbolKind = symbol.kind();

                        if let Ok(symbol_name) = symbol.name_bytes() {
                            if symbol_name == signature.name
                                && symbol.is_definition()
                                && ((signature.variant.is_function()
                                    && symbol_type == SymbolKind::Text)
                                    || (signature.variant.is_global()
                                        && symbol_type == SymbolKind::Data))
                                && !symbol.is_weak()
                            {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}
