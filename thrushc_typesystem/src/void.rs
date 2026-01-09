use crate::{Type, traits::VoidTypeExtensions};

impl VoidTypeExtensions for Type {
    fn contains_void_type(&self) -> bool {
        fn contains_void_type_inner_type(inner_type: &Type) -> bool {
            match inner_type {
                Type::Ptr(Some(inner_type), ..) => contains_void_type_inner_type(inner_type),
                Type::Const(inner_type, ..) => contains_void_type_inner_type(inner_type),
                Type::Array {
                    infered_type: Some((inner_type, _)),
                    ..
                } => contains_void_type_inner_type(inner_type),
                Type::Array {
                    base_type: inner_type,
                    ..
                } => contains_void_type_inner_type(inner_type),
                Type::Struct(_, field_types, _, _) => {
                    field_types.iter().any(contains_void_type_inner_type)
                }
                Type::FixedArray(inner_type, ..) => contains_void_type_inner_type(inner_type),
                Type::Fn(fields_types, return_type, ..) => {
                    fields_types.iter().any(contains_void_type_inner_type)
                        || contains_void_type_inner_type(return_type)
                }

                Type::Void(..) => true,

                _ => false,
            }
        }

        match self {
            Type::Ptr(Some(inner_type), ..) => contains_void_type_inner_type(inner_type),
            Type::Const(inner_type, ..) => contains_void_type_inner_type(inner_type),
            Type::Array {
                infered_type: Some((inner_type, _)),
                ..
            } => contains_void_type_inner_type(inner_type),
            Type::Array {
                base_type: inner_type,
                ..
            } => contains_void_type_inner_type(inner_type),
            Type::FixedArray(inner_type, ..) => contains_void_type_inner_type(inner_type),
            Type::Struct(_, field_types, _, _) => {
                field_types.iter().any(contains_void_type_inner_type)
            }
            Type::Fn(fields_types, return_type, ..) => {
                fields_types.iter().any(contains_void_type_inner_type)
                    || contains_void_type_inner_type(return_type)
            }

            _ => false,
        }
    }
}
