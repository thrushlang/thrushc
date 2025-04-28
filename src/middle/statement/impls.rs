use crate::middle::types::Type;

use super::{CustomTypeFields, traits::CustomTypeFieldsExtensions};

use crate::middle::instruction::Instruction;
use crate::middle::statement::traits::{
    AttributesExtensions, ConstructorExtensions, StructFieldsExtensions,
};

use crate::middle::statement::ThrushAttributes;
use crate::middle::statement::{Constructor, StructFields};

impl AttributesExtensions for ThrushAttributes<'_> {
    fn contain_ffi_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ffi_attribute())
    }

    fn contain_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    fn contain_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();
        Type::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl ConstructorExtensions for Constructor<'_> {
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.2.clone()).collect();
        Type::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

/*impl MappedHeapedPointersExtension<'_> for MappedHeapPointers<'_> {
    fn dealloc(
        &self,
        builder: &Builder,
        context: &Context,
        pointer: PointerValue,
        compiler_objects: &CompilerObjects,
    ) {
        self.iter()
            .filter(|mapped_pointer| !mapped_pointer.2)
            .for_each(|mapped_pointer| {
                let mapped_pointer_structure_name: &str = mapped_pointer.0;
                let mapped_pointer_index: u32 = mapped_pointer.1;

                let fields: &StructureFields = compiler_objects
                    .get_struct(mapped_pointer_structure_name)
                    .get_fields();

                let pointer_type: StructType = typegen::struct_type(context, fields);

                let target_pointer: PointerValue = builder
                    .build_struct_gep(pointer_type, pointer, mapped_pointer_index, "")
                    .unwrap();

                let loaded_target_pointer: PointerValue = builder
                    .build_load(target_pointer.get_type(), target_pointer, "")
                    .unwrap()
                    .into_pointer_value();

                let _ = builder.build_free(loaded_target_pointer);
            });
    }
}*/

impl CustomTypeFieldsExtensions for CustomTypeFields<'_> {
    fn get_type(&self) -> Type {
        Type::create_structure_type(String::new(), self)
    }
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Instruction::Integer(..), Instruction::Integer(..))
            | (Instruction::Float(..), Instruction::Float(..))
            | (Instruction::Str(..), Instruction::Str(..)) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}
