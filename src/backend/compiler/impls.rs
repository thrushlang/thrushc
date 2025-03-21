use super::{
    objects::CompilerObjects,
    traits::{AttributesExtensions, StructureBasics},
    types::{CompilerAttributes, Struct},
};

impl StructureBasics for Struct<'_> {
    fn contain_heaped_fields(&self, compiler_objects: &CompilerObjects) -> bool {
        self.iter().any(|field| {
            let is_structure: bool = field.1.is_struct_type() && !field.0.is_empty();

            let contain_another_heaped_fields: bool =
                if is_structure && compiler_objects.structs.contains_key(field.0) {
                    let struct_type: &Struct = compiler_objects.get_struct(field.0).unwrap();
                    struct_type.contain_heaped_fields(compiler_objects)
                } else {
                    false
                };

            is_structure && contain_another_heaped_fields
        })
    }
}

impl AttributesExtensions for CompilerAttributes<'_> {
    fn contain_ffi_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ffi_attribute())
    }

    fn contain_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }
}
