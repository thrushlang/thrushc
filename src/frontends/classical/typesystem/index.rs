use crate::frontends::classical::typesystem::{traits::IndexTypeExtensions, types::Type};

impl IndexTypeExtensions for Type {
    fn get_aprox_type(&self, depth: usize) -> &Type {
        if depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(element_type, _) => element_type.get_aprox_type(depth),
            Type::Array(element_type) => element_type.get_aprox_type(depth),
            Type::Mut(inner_type) => inner_type.get_aprox_type(depth),
            Type::Const(inner_type) => inner_type.get_aprox_type(depth),
            Type::Ptr(Some(inner_type)) => inner_type.get_aprox_type(depth - 1),
            Type::Struct(..) => self,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Char
            | Type::Addr
            | Type::Void
            | Type::Ptr(None)
            | Type::Fn(..) => self,
        }
    }
}
