use crate::front_end::typesystem::{traits::TypeArrayEntensions, types::Type};

impl TypeArrayEntensions for Type {
    #[inline]
    fn get_array_base_type(&self) -> &Type {
        if let Type::Array(inner, ..) = self {
            return inner;
        }

        if let Type::Ptr(Some(inner)) = self {
            return inner.get_array_base_type();
        }

        if let Type::Const(inner) = self {
            return inner.get_array_base_type();
        }

        self
    }

    #[inline]
    fn get_array_type_herarchy(&self) -> u8 {
        match self {
            Type::Void => 0,

            Type::Bool => 1,
            Type::Char => 2,

            Type::S8 => 4,
            Type::S16 => 5,
            Type::S32 => 6,
            Type::S64 => 7,

            Type::U8 => 8,
            Type::U16 => 9,
            Type::U32 => 10,
            Type::U64 => 11,
            Type::U128 => 12,

            Type::F32 => 13,
            Type::F64 => 14,
            Type::F128 => 15,
            Type::FX8680 => 16,
            Type::FPPC128 => 17,

            Type::Const(subtype) => subtype.get_array_type_herarchy(),

            Type::Addr => 18,
            Type::NullPtr => 19,
            Type::Ptr(Some(subtype)) => subtype.get_array_type_herarchy(),
            Type::Ptr(None) => 20,

            Type::FixedArray(..) => 21,
            Type::Array(..) => 22,
            Type::Struct(..) => 23,

            Type::Fn(..) => 24,
        }
    }
}
