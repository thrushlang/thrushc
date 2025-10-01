use crate::frontends::classical::typesystem::{traits::CastTypeExtensions, types::Type};

impl CastTypeExtensions for Type {
    #[inline]
    fn narrowing(&self) -> Type {
        match self {
            Type::U8 => Type::S8,
            Type::U16 => Type::S16,
            Type::U32 => Type::S32,
            Type::U64 => Type::S64,

            Type::S8 => Type::U8,
            Type::S16 => Type::U16,
            Type::S32 => Type::U32,
            Type::S64 => Type::U64,

            _ => self.clone(),
        }
    }

    #[inline]
    fn precompute(&self, other: &Type) -> Type {
        match (self, other) {
            (Type::S64, _) | (_, Type::S64) => Type::S64,
            (Type::S32, _) | (_, Type::S32) => Type::S32,
            (Type::S16, _) | (_, Type::S16) => Type::S16,
            (Type::S8, _) | (_, Type::S8) => Type::S8,

            (Type::U64, _) | (_, Type::U64) => Type::U64,
            (Type::U32, _) | (_, Type::U32) => Type::U32,
            (Type::U16, _) | (_, Type::U16) => Type::U16,
            (Type::U8, _) | (_, Type::U8) => Type::U8,

            (Type::F64, _) | (_, Type::F64) => Type::F64,
            (Type::F32, _) | (_, Type::F32) => Type::F32,

            (Type::FX8680, _) | (_, Type::FX8680) => Type::FX8680,

            (Type::Const(lhs), Type::Const(rhs)) => lhs.precompute(rhs),

            _ => self.clone(),
        }
    }
}
