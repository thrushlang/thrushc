use crate::front_end::typesystem::traits::CastTypeExtensions;
use crate::front_end::typesystem::types::Type;

impl CastTypeExtensions for Type {
    #[inline]
    fn narrowing(&self) -> Type {
        match self {
            Type::U8(span) => Type::S8(*span),
            Type::U16(span) => Type::S16(*span),
            Type::U32(span) => Type::S32(*span),
            Type::U64(span) => Type::S64(*span),
            Type::USize(span) => Type::SSize(*span),

            Type::S8(span) => Type::U8(*span),
            Type::S16(span) => Type::U16(*span),
            Type::S32(span) => Type::U32(*span),
            Type::S64(span) => Type::U64(*span),
            Type::SSize(span) => Type::USize(*span),

            _ => self.clone(),
        }
    }
}
