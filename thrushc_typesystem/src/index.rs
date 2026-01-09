use crate::{
    Type,
    traits::{IndexExtensions, TypeExtensions, TypeIsExtensions},
};

impl IndexExtensions for Type {
    fn calculate_index_type(&self, depth: usize) -> &Type {
        if depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(inner_type, ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Array {
                infered_type: Some((infered_type, 0)),
                ..
            } => infered_type.get_type_with_depth(depth),
            Type::Array { base_type, .. } => base_type.get_type_with_depth(depth - 1),
            Type::Const(inner_type, ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Ptr(Some(inner_type), ..) if !inner_type.is_ptr_like_type() => {
                inner_type.get_type_with_depth(depth)
            }
            Type::Ptr(Some(inner_type), ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Struct(..) => self,
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::SSize(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::F32(..)
            | Type::F64(..)
            | Type::F128(..)
            | Type::FX8680(..)
            | Type::FPPC128(..)
            | Type::Bool(..)
            | Type::Char(..)
            | Type::Addr(..)
            | Type::Void(..)
            | Type::Ptr(None, ..)
            | Type::Fn(..) => self,
        }
    }
}
