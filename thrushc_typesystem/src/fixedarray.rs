use crate::{Type, traits::TypeFixedArrayEntensions};

impl TypeFixedArrayEntensions for Type {
    #[inline]
    fn get_fixed_array_base_type(&self) -> Type {
        if let Type::FixedArray(inner, ..) = self {
            return *(*inner).clone();
        }

        if let Type::Ptr(Some(inner), ..) = self {
            return inner.get_fixed_array_base_type();
        }

        if let Type::Const(inner, ..) = self {
            return inner.get_fixed_array_base_type();
        }

        self.clone()
    }

    #[inline]
    fn get_fixed_array_type_herarchy(&self) -> u8 {
        match self {
            Type::Bool(..) => 1,
            Type::Char(..) => 2,

            Type::U8(..) => 3,
            Type::U16(..) => 4,
            Type::U32(..) => 5,
            Type::U64(..) => 6,
            Type::U128(..) => 7,
            Type::USize(..) => 8,

            Type::S8(..) => 9,
            Type::S16(..) => 10,
            Type::S32(..) => 11,
            Type::S64(..) => 12,
            Type::SSize(..) => 13,

            Type::F32(..) => 15,
            Type::F64(..) => 16,
            Type::F128(..) => 17,
            Type::FX8680(..) => 18,
            Type::FPPC128(..) => 19,

            Type::Const(subtype, ..) => subtype.get_fixed_array_type_herarchy(),

            Type::Addr(..) => 20,
            Type::Ptr(Some(subtype), ..) => subtype.get_fixed_array_type_herarchy(),
            Type::Ptr(None, ..) => 21,

            Type::Array { .. } => 22,
            Type::FixedArray(..) => 23,
            Type::Struct(..) => 24,

            Type::Fn(..) => 25,

            Type::Void(..) => 25,
        }
    }
}
