use crate::front_end::typesystem::{traits::TypeFixedArrayEntensions, types::Type};

impl TypeFixedArrayEntensions for Type {
    #[inline]
    fn get_fixed_array_base_type(&self) -> &Type {
        if let Type::FixedArray(inner, ..) = self {
            return inner;
        }

        if let Type::Ptr(Some(inner)) = self {
            return inner.get_fixed_array_base_type();
        }

        if let Type::Const(inner) = self {
            return inner.get_fixed_array_base_type();
        }

        self
    }

    #[inline]
    fn get_fixed_array_type_herarchy(&self) -> u8 {
        match self {
            Type::Void => 0,

            Type::Bool => 1,
            Type::Char => 2,

            Type::S8 => 4,
            Type::S16 => 5,
            Type::S32 => 6,
            Type::S64 => 7,
            Type::SSize => 8,

            Type::U8 => 9,
            Type::U16 => 10,
            Type::U32 => 11,
            Type::U64 => 12,
            Type::U128 => 13,
            Type::USize => 14,

            Type::F32 => 15,
            Type::F64 => 16,
            Type::F128 => 17,
            Type::FX8680 => 18,
            Type::FPPC128 => 19,

            Type::Const(subtype) => subtype.get_fixed_array_type_herarchy(),

            Type::Addr => 20,
            Type::NullPtr => 21,
            Type::Ptr(Some(subtype)) => subtype.get_fixed_array_type_herarchy(),
            Type::Ptr(None) => 22,

            Type::FixedArray(..) => 23,
            Type::Array(..) => 24,
            Type::Struct(..) => 25,

            Type::Fn(..) => 26,
        }
    }
}
